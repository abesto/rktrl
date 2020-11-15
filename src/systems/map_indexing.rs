use legion::{system, world::SubWorld, Entity, IntoQuery};

use crate::{components::*, resources::*};

#[system]
#[read_component(Entity)]
#[read_component(Position)]
#[read_component(Player)]
#[read_component(BlocksTile)]
pub fn map_indexing(#[resource] map: &mut Map, world: &SubWorld) {
    map.populate_blocked();
    map.clear_content_index();

    <(Entity, &Position, Option<&Player>, Option<&BlocksTile>)>::query().for_each(
        world,
        |(entity, position, player, blocks)| {
            map.add_tile_content(*position, *entity);

            if player.is_none() && blocks.is_some() {
                map.block(*position)
            }
        },
    );
}
