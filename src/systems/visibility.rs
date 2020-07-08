use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(VisibilitySystemData(
    write_storage(Viewshed),
    read_storage(Position),
    read_expect(Map)
));

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = VisibilitySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let map = &*data.map;
        for (viewshed, pos) in (&mut data.viewsheds, &data.positions).join() {
            if !viewshed.dirty {
                continue;
            }
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles =
                field_of_view(Point::new(pos.x, pos.y), viewshed.range.into(), map)
                    .iter()
                    .map(|p| Position::from(*p))
                    .filter(|p| map.contains(*p))
                    .collect();
            viewshed.revealed_tiles.extend(&viewshed.visible_tiles);
            viewshed.dirty = false;
        }
    }
}
