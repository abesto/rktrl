use crate::{
    components::{position::Position, viewshed::Viewshed},
    resources::map::Map,
};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use std::convert::TryInto;

#[derive(SystemData)]
pub struct VisibilitySystemData<'a> {
    viewshed: WriteStorage<'a, Viewshed>,
    position: ReadStorage<'a, Position>,

    map: Read<'a, Map>,
}

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = VisibilitySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let map = &*data.map;
        for (viewshed, pos) in (&mut data.viewshed, &data.position).join() {
            if !viewshed.dirty {
                continue;
            }
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles =
                field_of_view(Point::new(pos.x, pos.y), viewshed.range.into(), map)
                    .iter()
                    .filter_map(|p| p.try_into().ok())
                    .filter(|p| map.contains(*p))
                    .collect();
            viewshed.revealed_tiles.extend(&viewshed.visible_tiles);
            viewshed.dirty = false;
        }
    }
}
