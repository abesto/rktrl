use bracket_lib::prelude::RandomNumberGenerator;

use crate::components::Position;
use crate::mapgen::simple::SimpleMapBuilder;
use crate::resources::Map;
use crate::systems::prelude::CommandBuffer;

mod common;
mod simple;
pub mod spawner;

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
}

pub fn random_builder<'a>(width: i32, height: i32, new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(width, height, new_depth))
}
