use crate::systems::prelude::*;

use std::collections::VecDeque;

use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;

use super::{common::*, MapBuilder, SnapshotManager};
use crate::mapgen::spawner::spawn_area;

pub struct DrunkardsWalkMapBuilder {
    map: Map,
    snapshot_manager: SnapshotManager,
    // We just pick a random empty spot, so need to store it for it to be stable
    cached_starting_position: Option<Position>,
}

impl MapBuilder for DrunkardsWalkMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        self.take_snapshot();
        let threshold = 0.4;
        let mut to_dig = (((self.map.width * self.map.height) as f64) * threshold) as u32;

        let mut pos = random_position_with_tile(TileType::Wall, &self.map, rng);
        while to_dig > 0 {
            let direction = Heading::iter().choose(rng.get_rng()).unwrap();
            let pos_candidate = pos + Vector::unit(direction);
            if !self.map.contains(pos_candidate) {
                continue;
            }
            pos = pos_candidate;

            if self.map[&pos] != TileType::Wall {
                continue;
            }
            to_dig -= 1;
            self.map[&pos] = TileType::Floor;

            if to_dig % 100 == 0 {
                self.take_snapshot();
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

impl DrunkardsWalkMapBuilder {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        DrunkardsWalkMapBuilder {
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
            cached_starting_position: None,
        }
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }
}
