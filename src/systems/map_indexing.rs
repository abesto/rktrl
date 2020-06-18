use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{blocks_tile::BlocksTile, player::Player, position::Position},
    resources::map::Map,
};

#[derive(SystemData)]
pub struct MapIndexingSystemData<'a> {
    position: ReadStorage<'a, Position>,
    blocks_tile: ReadStorage<'a, BlocksTile>,
    player: ReadStorage<'a, Player>,

    map: Write<'a, Map>,
}

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = MapIndexingSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.map.populate_blocked();
        for (position, player, _blocks) in
            (&data.position, data.player.maybe(), &data.blocks_tile).join()
        {
            if player == None {
                data.map.block(*position)
            }
        }
    }
}
