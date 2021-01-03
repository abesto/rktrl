use crate::systems::prelude::*;
use petgraph::Graph;
use rand::prelude::SliceRandom;
use std::cmp::{max, min, Ordering};
use std::collections::VecDeque;

use super::{common::*, MapBuilder, SnapshotManager};

pub struct BspConfig {
    /// 0..=1; higher values lead to more varied room aspect ratios
    pub subdivision_variance: f64,
    pub depth: i32,

    pub min_room_width: i32,
    pub max_room_width: i32,

    pub min_room_height: i32,
    pub max_room_height: i32,

    /// Minimum number of wall tiles between the edge of the region and the start of the room.
    /// Value is *per side*.
    pub min_padding: i32,
    /// Maximum number of wall tiles between the edge of the region and the start of the room.
    /// Value is *per side*.
    pub max_padding: i32,
}

impl BspConfig {
    pub fn dungeon() -> Self {
        BspConfig {
            subdivision_variance: 0.2,
            depth: 6,
            min_room_width: 6,
            max_room_width: 10,
            min_room_height: 6,
            max_room_height: 10,
            max_padding: 9000, // Arbitrarily large number. Not maxint because that leads to overflow.
            min_padding: 2,
        }
    }

    pub fn interior() -> Self {
        BspConfig {
            subdivision_variance: 0.2,
            depth: 5,
            min_room_width: 6,
            max_room_width: 9000,
            min_room_height: 6,
            max_room_height: 9000,
            max_padding: 0,
            min_padding: 0,
        }
    }

    fn min_region_width(&self) -> i32 {
        self.min_room_width + self.min_padding * 2
    }

    fn min_region_height(&self) -> i32 {
        self.min_room_height + self.min_padding * 2
    }

    fn subdivision_min(&self) -> f64 {
        0.5 - self.subdivision_variance / 2.0
    }

    fn subdivision_max(&self) -> f64 {
        0.5 + self.subdivision_variance / 2.0
    }
}

pub struct BspDungeonMapBuilder {
    config: BspConfig,
    rooms: Vec<Rect>,
    map: Map,
    snapshot_manager: SnapshotManager,
}

impl MapBuilder for BspDungeonMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        let mut graph = Graph::<Rect, ()>::new();
        let root = graph.add_node(Rect::with_size(
            0,
            0,
            self.map.width - 1,
            self.map.height - 1,
        ));
        let mut leaves = vec![root];

        // Generate space partition
        let mut max_depth = 0;
        for depth in 1..self.config.depth + 1 {
            leaves = leaves
                .iter()
                .flat_map(|&leaf| {
                    let leaf_rect = graph.node_weight(leaf).unwrap();
                    let mut a_rect;
                    let mut b_rect;

                    a_rect = *leaf_rect;
                    b_rect = *leaf_rect;
                    let position =
                        rng.range(self.config.subdivision_min(), self.config.subdivision_max());
                    if rng.roll_dice(1, 2) == 1 {
                        a_rect.x2 -= (a_rect.width() as f64 * position).round() as i32;
                        b_rect.x1 = a_rect.x2;
                    } else {
                        a_rect.y2 -= (a_rect.height() as f64 * position).round() as i32;
                        b_rect.y1 = a_rect.y2;
                    }

                    if a_rect.width() < self.config.min_region_width()
                        || b_rect.width() < self.config.min_region_width()
                        || a_rect.height() < self.config.min_region_height()
                        || b_rect.height() < self.config.min_region_height()
                    {
                        vec![leaf]
                    } else {
                        let a = graph.add_node(a_rect);
                        let b = graph.add_node(b_rect);
                        graph.add_edge(leaf, a, ());
                        graph.add_edge(leaf, b, ());
                        max_depth = max(max_depth, depth);
                        vec![a, b]
                    }
                })
                .collect();

            if cfg!(feature = "visualize-mapgen") {
                self.take_subdivision_snapshot(
                    leaves
                        .iter()
                        .map(|index| *graph.node_weight(*index).unwrap())
                        .collect(),
                );
            }
        }

        // Create room in each partition
        for leaf in leaves {
            let partition = graph.node_weight(leaf).unwrap();

            // Generate random room width based on config
            let min_width = max(
                self.config.min_room_width,
                partition.width() - self.config.max_padding * 2,
            );
            let max_width = min(
                partition.width() - self.config.min_padding * 2,
                self.config.max_room_width,
            );
            let width = match min_width.cmp(&max_width) {
                Ordering::Equal => min_width,
                Ordering::Less => rng.range(min_width, max_width),
                _ => unreachable!(),
            };

            // Generate random room left-edge, based on config and width
            let min_x1 = partition.x1 + self.config.min_padding;
            let max_x1 = partition.x2 - width - self.config.min_padding;
            let x1 = match min_x1.cmp(&max_x1) {
                Ordering::Equal => min_x1,
                Ordering::Less => rng.range(min_x1, max_x1),
                _ => unreachable!(),
            };

            // Generate random room height based on config
            let min_height = max(
                self.config.min_room_height,
                partition.height() - self.config.max_padding * 2,
            );
            let max_height = min(
                partition.height() - self.config.min_padding * 2,
                self.config.max_room_width,
            );
            let height = match min_height.cmp(&max_height) {
                Ordering::Equal => min_height,
                Ordering::Less => rng.range(min_height, max_height),
                _ => unreachable!(),
            };

            // Generate random room top-edge, based on config and height
            let min_y1 = partition.y1 + self.config.min_padding;
            let may_y1 = partition.y2 - height - self.config.min_padding;
            let y1 = match min_y1.cmp(&may_y1) {
                Ordering::Equal => min_y1,
                Ordering::Less => rng.range(min_y1, may_y1),
                _ => unreachable!(),
            };

            let room = Rect::with_size(x1, y1, width, height);
            apply_room_to_map(&room, &mut self.map);
            self.add_walls(&room);
            self.rooms.push(room);
        }
        self.take_snapshot();

        // Add corridors
        for depth in (0..max_depth).rev() {
            let mut parents = vec![root];

            // Find nodes at `depth`
            for _ in 0..depth {
                parents = parents
                    .iter()
                    .flat_map(|&index| graph.neighbors(index))
                    .collect();
            }

            // Connect the children of each node at `depth`
            for parent in parents {
                let children = graph.neighbors(parent).collect::<Vec<_>>();
                if children.len() < 2 {
                    continue;
                }
                for i in 0..children.len() - 1 {
                    connect_regions(
                        *graph.node_weight(children[i]).unwrap(),
                        *graph.node_weight(children[i + 1]).unwrap(),
                        &mut self.map,
                        rng,
                    );
                    self.take_snapshot();
                }
            }
        }

        // Shuffle rooms so that starting, stairs positions are interesting
        self.rooms.shuffle(rng.get_rng());

        // Place stairs down in "last" room
        let last_room = self.rooms.last().unwrap();
        self.map[&last_room.center().into()] = TileType::DownStairs;
    }

    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator) {
        for room in self.rooms.iter().skip(1) {
            super::spawner::room(rng, room, self.map.depth, commands);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.rooms[0].center().into()
    }

    fn get_snapshots(&self) -> VecDeque<Map> {
        self.snapshot_manager.get_snapshots()
    }
}

impl BspDungeonMapBuilder {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32, config: BspConfig) -> Self {
        BspDungeonMapBuilder {
            rooms: vec![],
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
            config,
        }
    }

    fn clear(&mut self, tile: TileType) {
        for x in 0..self.map.width {
            for y in 0..self.map.height {
                self.map[&Position::new(x, y)] = tile;
            }
        }
    }

    fn add_walls(&mut self, rect: &Rect) {
        for x in rect.x1..=rect.x2 {
            self.map[&Position::new(x, rect.y1)] = TileType::Wall;
            self.map[&Position::new(x, rect.y2)] = TileType::Wall;
        }
        for y in rect.y1..=rect.y2 {
            self.map[&Position::new(rect.x1, y)] = TileType::Wall;
            self.map[&Position::new(rect.x2, y)] = TileType::Wall;
        }
    }

    fn take_subdivision_snapshot(&mut self, rects: Vec<Rect>) {
        self.clear(TileType::Floor);
        for rect in rects {
            self.add_walls(&rect);
        }
        self.take_snapshot();
        self.clear(TileType::Wall);
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }
}
