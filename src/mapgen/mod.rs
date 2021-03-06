use bracket_lib::prelude::RandomNumberGenerator;
use std::collections::VecDeque;

use crate::components::Position;
use crate::mapgen::bsp::{BspConfig, BspMapBuilder};
use crate::mapgen::cellular_automata::{CellularAutomataMapBuilder, DefaultCellularAutomataConfig};
use crate::mapgen::drunkards_walk::DrunkardsWalkMapBuilder;
use crate::mapgen::maze::MazeMapBuilder;
use crate::mapgen::simple::SimpleMapBuilder;
use crate::resources::Map;
use crate::systems::prelude::CommandBuffer;

mod bsp;
mod cellular_automata;
mod common;
mod drunkards_walk;
mod maze;
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
        if cfg!(feature = "visualize-mapgen") {
            self.snapshots.push_back(map);
        }
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

pub fn random_builder(
    rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
    new_depth: i32,
) -> Box<dyn MapBuilder> {
    match rng.roll_dice(1, 6) {
        1 => Box::new(BspMapBuilder::new(
            width,
            height,
            new_depth,
            BspConfig::dungeon(),
        )),
        2 => Box::new(BspMapBuilder::new(
            width,
            height,
            new_depth,
            BspConfig::interior(),
        )),
        3 => Box::new(CellularAutomataMapBuilder::new(
            width,
            height,
            new_depth,
            Box::new(DefaultCellularAutomataConfig),
        )),
        4 => Box::new(DrunkardsWalkMapBuilder::new(width, height, new_depth)),
        5 => Box::new(MazeMapBuilder::new(width, height, new_depth)),
        _ => Box::new(SimpleMapBuilder::new(width, height, new_depth)),
    }
}
