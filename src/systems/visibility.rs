use crate::systems::prelude::*;

#[system(for_each)]
pub fn visibility(viewshed: &mut Viewshed, pos: &Position, #[resource] map: &Map) {
    if !viewshed.dirty {
        return;
    }
    viewshed.visible_tiles.clear();
    viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range.into(), map)
        .iter()
        .map(|p| Position::from(*p))
        .filter(|p| map.contains(*p))
        .collect();
    viewshed.revealed_tiles.extend(&viewshed.visible_tiles);
    viewshed.dirty = false;
}
