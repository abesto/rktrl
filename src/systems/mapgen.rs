use crate::components::{position::Position, renderable::Renderable};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

#[derive(SystemData)]
pub struct MapgenSystemData<'a> {
    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,

    entity: Entities<'a>,
}

pub struct MapgenSystem;

impl<'a> System<'a> for MapgenSystem {
    type SystemData = MapgenSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.entity
            .build_entity()
            .with(Position { x: 40, y: 25 }, &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437('@'),
                    fg: RGB::named(YELLOW),
                    bg: RGB::named(BLACK),
                },
                &mut data.renderable,
            )
            .build();

        for i in 0..10 {
            data.entity
                .build_entity()
                .with(Position { x: i * 7, y: 20 }, &mut data.position)
                .with(
                    Renderable {
                        glyph: to_cp437('☺'),
                        fg: RGB::named(RED),
                        bg: RGB::named(BLACK),
                    },
                    &mut data.renderable,
                )
                .build();
        }
    }
}

impl MapgenSystem {
    pub fn new() -> MapgenSystem {
        return MapgenSystem {};
    }
}
