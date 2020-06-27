use shipyard::*;

use crate::{
    components::{blocks_tile::BlocksTile, player::Player, position::Position},
    resources::map::Map,
};

pub fn map_indexing(
    ref entities: EntitiesView,
    ref positions: View<Position>,
    ref players: View<Player>,
    ref tile_blockers: View<BlocksTile>,
    ref mut map: UniqueViewMut<Map>,
) {
    map.populate_blocked();
    map.clear_content_index();

    for (entity, position) in positions.iter().with_id() {
        map.add_tile_content(*position, entity.clone());

        if !players.contains(entity.clone()) && tile_blockers.contains(entity.clone()) {
            map.block(*position)
        }
    }
}
