use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(MapIndexingSystemData(
    entities,
    read_storage(Position, BlocksTile, Player),
    write_expect(Map)
));

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = MapIndexingSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.map.populate_blocked();
        data.map.clear_content_index();

        for (entity, position, player, blocks) in (
            &data.entities,
            &data.positions,
            data.players.maybe(),
            data.blocks_tiles.maybe(),
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
