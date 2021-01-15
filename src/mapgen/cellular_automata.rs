use crate::systems::prelude::*;

use std::collections::VecDeque;

use super::{common::*, MapBuilder, SnapshotManager};
use crate::mapgen::spawner::spawn_area;

pub trait CellularAutomataConfig {
    /// For each position: is the starting tile a wall?
    fn init(&self, rng: &mut RandomNumberGenerator) -> bool;

    /// Given the previous `map` state, is the tile at `position` a wall?
    /// `step` goes from 1 to `self.steps()`
    fn rule(&self, pos: Position, map: &Map, step: i32) -> bool;

    /// How many iterations should the automata run?
    fn steps(&self) -> i32;
    /// After all the steps are done, take care of any required post-processing
    /// (like filling in non-contiguous areas)
    /// Return `false` to accept the map; return `false` to restart map generation
    fn postprocess(&self, map: &mut Map, rng: &mut RandomNumberGenerator) -> bool;
}

fn walls_within_radius(n: i32, center: Position, map: &Map) -> i32 {
    let mut wall_count = 0;
    for dx in -n..=n {
        for dy in -n..=n {
            if dx == 0 && dy == 0 {
                continue;
            }
            let pos = center + Vector::new(dx, dy);
            if map.contains(pos) && map[&pos] == TileType::Wall {
                wall_count += 1;
            }
        }
    }
    wall_count
}

/// Implement the method described at http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
///
/// Specifically:
/// Winit(p) = rand(0,100) < 40
/// Repeat 4: W'(p) = R1(p) >= 5 || R2(p) <= 2
/// Repeat 3: W'(p) = R1(p) >= 5
///
/// Then: pick a random empty space, flood fill. Anything outside the flood fill becomes a wall.
/// Accept the map if, at the end of this, >=30% of the map is non-wall.
pub struct DefaultCellularAutomataConfig;

impl CellularAutomataConfig for DefaultCellularAutomataConfig {
    fn init(&self, rng: &mut RandomNumberGenerator) -> bool {
        rng.roll_dice(1, 100) < 40
    }

    fn rule(&self, pos: Position, map: &Map, step: i32) -> bool {
        if step <= 4 {
            walls_within_radius(1, pos, map) >= 5 || walls_within_radius(2, pos, map) <= 2
        } else {
            walls_within_radius(1, pos, map) >= 5
        }
    }

    fn steps(&self) -> i32 {
        7
    }

    fn postprocess(&self, map: &mut Map, rng: &mut RandomNumberGenerator) -> bool {
        // Add walls around the edges of the map so that it doesn't just cut off weirdly
        walls_around(&Rect::with_size(0, 0, map.width - 1, map.height - 1), map);

        // Find a random empty spot
        let random_floor = random_position_with_tile(TileType::Floor, map, rng);

        // Find the area connected with the spot picked. Fill in the rest.
        // If it's less than $threshold of the size of the map, then reject the map; otherwise accept.
        let connected_region = remove_unreachable_areas(&random_floor, map);
        let threshold = ((map.width * map.height) as f32 * 0.30).round() as usize;
        connected_region.len() >= threshold
    }
}

pub struct CellularAutomataMapBuilder {
    config: Box<dyn CellularAutomataConfig>,
    map: Map,
    snapshot_manager: SnapshotManager,
    // We just pick a random empty spot, so need to store it for it to be stable
    cached_starting_position: Option<Position>,
}

impl MapBuilder for CellularAutomataMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        loop {
            // Initialize each tile
            for pos in self.map.position_set() {
                self.map[&pos] = Self::bool_to_tile(self.config.init(rng));
            }
            self.take_snapshot();

            // Apply cellular automata rules
            for step in 1..=self.config.steps() {
                let old_map = self.map.clone();
                for (pos, _) in &old_map {
                    self.map[&pos] = Self::bool_to_tile(self.config.rule(pos, &old_map, step));
                }
                self.take_snapshot();
            }

            // Apply any postprocessing
            let accept = self.config.postprocess(&mut self.map, rng);
            self.take_snapshot();

            // If we like the map, then we're done!
            if accept {
                break;
            }
        }

        // Pick a random starting position
        let starting_position = random_position_with_tile(TileType::Floor, &self.map, rng);
        self.cached_starting_position = Some(starting_position);

        // Pick a random position for the downwards staircase that's pretty far from the player
        let dijsktra_map = bracket_lib::prelude::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &[self.map.pos_idx(starting_position)],
            &self.map,
            200.0,
        );
        let furthest_reachable_tiles = find_furthest_reachable_tiles(&self.map, &dijsktra_map, 20);
        let stairs_down_pos = rng.random_slice_entry(&furthest_reachable_tiles).unwrap();
        self.map[stairs_down_pos] = TileType::DownStairs;
        self.take_snapshot();
    }

    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator) {
        // Generate spawning areas
        let mut areas = generate_voronoi_spawn_regions(&self.map, rng);

        // Generate entities in each area
        for area in areas.values_mut() {
            spawn_area(rng, area, self.map.depth, commands);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.cached_starting_position.unwrap()
    }

    fn get_snapshots(&self) -> VecDeque<Map> {
        self.snapshot_manager.get_snapshots()
    }
}

impl CellularAutomataMapBuilder {
    #[must_use]
    pub fn new(
        width: i32,
        height: i32,
        depth: i32,
        config: Box<dyn CellularAutomataConfig>,
    ) -> Self {
        CellularAutomataMapBuilder {
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
            config,
            cached_starting_position: None,
        }
    }

    fn bool_to_tile(is_wall: bool) -> TileType {
        if is_wall {
            TileType::Wall
        } else {
            TileType::Floor
        }
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }
}
