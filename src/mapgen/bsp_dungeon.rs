use crate::systems::prelude::*;
use petgraph::Graph;
use std::cmp::max;
use std::collections::VecDeque;

use super::{common::*, MapBuilder, SnapshotManager};

pub struct BspDungeonMapBuilder {
    rooms: Vec<Rect>,
    map: Map,
    snapshot_manager: SnapshotManager,
}

impl MapBuilder for BspDungeonMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        let mut graph = Graph::<Rect, ()>::new();
        let root = graph.add_node(Rect::with_size(
            1,
            1,
            self.map.width - 3,
            self.map.height - 3,
        ));
        let mut leaves = vec![root];

        // Generate space partition
        let mut max_depth = 0;
        for depth in 1..6 {
            leaves = leaves
                .iter()
                .flat_map(|&leaf| {
                    let leaf_rect = graph.node_weight(leaf).unwrap();
                    let mut a_rect;
                    let mut b_rect;

                    a_rect = *leaf_rect;
                    b_rect = *leaf_rect;
                    let position = rng.range(0.4, 0.6);
                    if rng.roll_dice(1, 2) == 1 {
                        a_rect.x2 -= (a_rect.width() as f64 * position).round() as i32;
                        b_rect.x1 = a_rect.x2;
                    } else {
                        a_rect.y2 -= (a_rect.height() as f64 * position).round() as i32;
                        b_rect.y1 = a_rect.y2;
                    }

                    if a_rect.width() < 10
                        || b_rect.width() < 10
                        || a_rect.height() < 8
                        || b_rect.height() < 8
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

            let width = rng.range(5, partition.width() - 2);
            let x1 = rng.range(partition.x1 + 1, partition.x2 - width);

            let height = rng.range(5, partition.height() - 2);
            let y1 = rng.range(partition.y1 + 1, partition.y2 - height);

            let room = Rect::with_exact(x1, y1, x1 + width, y1 + height);
            apply_room_to_map(&room, &mut self.map);
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
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        BspDungeonMapBuilder {
            rooms: vec![],
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
        }
    }

    fn clear(&mut self, tile: TileType) {
        for x in 0..self.map.width {
            for y in 0..self.map.height {
                self.map[&Position::new(x, y)] = tile;
            }
        }
    }

    fn take_subdivision_snapshot(&mut self, rects: Vec<Rect>) {
        self.clear(TileType::Floor);
        for rect in rects {
            for x in rect.x1..=rect.x2 {
                self.map[&Position::new(x, rect.y1)] = TileType::Wall;
                self.map[&Position::new(x, rect.y2)] = TileType::Wall;
            }
            for y in rect.y1..=rect.y2 {
                self.map[&Position::new(rect.x1, y)] = TileType::Wall;
                self.map[&Position::new(rect.x2, y)] = TileType::Wall;
            }
        }
        self.take_snapshot();
        self.clear(TileType::Wall);
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }
}
