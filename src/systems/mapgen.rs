use crate::{
    components::{player::Player, position::Position, renderable::Renderable},
    resources::map::{Map, TileType},
};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

#[derive(SystemData)]
pub struct MapgenSystemData<'a> {
    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,
    player: WriteStorage<'a, Player>,

    map: Write<'a, Map>,
    entity: Entities<'a>,
}

pub struct MapgenSystem;

impl<'a> System<'a> for MapgenSystem {
    type SystemData = MapgenSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        self.gen_map(&mut data);
        self.gen_mobs(&mut data);
    }
}

impl MapgenSystem {
    pub fn new() -> MapgenSystem {
        MapgenSystem {}
    }

    fn gen_mobs(&mut self, data: &mut MapgenSystemData) {
        data.entity
            .build_entity()
            .with(Position::new(40, 25), &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437('@'),
                    fg: RGB::named(YELLOW),
                    bg: RGB::named(BLACK),
                },
                &mut data.renderable,
            )
            .with(Player::new(), &mut data.player)
            .build();

        for i in 0..10 {
            data.entity
                .build_entity()
                .with(Position::new(i * 7, 20), &mut data.position)
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

    fn gen_map(&mut self, data: &mut MapgenSystemData) {
        let map = &mut data.map;

        // Make the boundaries walls
        for x in 0..80 {
            map[&Position::new(x, 0)] = TileType::Wall;
            map[&Position::new(x, 49)] = TileType::Wall;
        }
        for y in 0..50 {
            map[&Position::new(0, y)] = TileType::Wall;
            map[&Position::new(79, y)] = TileType::Wall;
        }

        // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
        // First, obtain the thread-local RNG:
        let mut rng = RandomNumberGenerator::new();

        for _i in 0..400 {
            let pos = Position::new(rng.roll_dice(1, 79), rng.roll_dice(1, 49));
            let exclude = Position::new(40, 25);
            if pos != exclude {
                map[&pos] = TileType::Wall;
            }
        }
    }
}
