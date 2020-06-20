use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{blocks_tile::BlocksTile, player::Player, position::Position},
    resources::map::Map,
};

#[derive(SystemData)]
pub struct MapIndexingSystemData<'a> {
    entities: Entities<'a>,
    position: ReadStorage<'a, Position>,
    blocks_tile: ReadStorage<'a, BlocksTile>,
    player: ReadStorage<'a, Player>,

    map: WriteExpect<'a, Map>,
}

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = MapIndexingSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.map.populate_blocked();
        data.map.clear_content_index();

        for (entity, position, player, blocks) in (
            &data.entities,
            &data.position,
            data.player.maybe(),
            data.blocks_tile.maybe(),
        )
            .join()
        {
            data.map.add_tile_content(*position, entity);

            if player.is_none() && blocks.is_some() {
                data.map.block(*position)
            }
        }
    }
}
