use bracket_lib::prelude::RandomNumberGenerator;
use std::collections::VecDeque;

use crate::components::Position;
use crate::mapgen::simple::SimpleMapBuilder;
use crate::resources::Map;
use crate::systems::prelude::CommandBuffer;

mod common;
mod simple;
pub mod spawner;

struct SnapshotManager {
    snapshots: VecDeque<Map>,
}

impl SnapshotManager {
    #[must_use]
    fn new() -> Self {
        SnapshotManager {
            snapshots: VecDeque::new(),
        }
    }

    fn record_snapshot(&mut self, map: Map) {
        #[cfg(feature = "visualize-mapgen")]
        self.snapshots.push_back(map);
    }

    fn get_snapshots(&self) -> VecDeque<Map> {
        self.snapshots.clone()
    }
}

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshots(&self) -> VecDeque<Map>;
}

pub fn random_builder(width: i32, height: i32, new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(width, height, new_depth))
}
