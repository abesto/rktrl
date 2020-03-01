use crate::{
    components::{player::Player, position::Position, renderable::Renderable},
    lib::rect::Rect,
    resources::map::{Map, TileType},
};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use std::cmp::{max, min};

#[derive(SystemData)]
pub struct MapgenSystemData<'a> {
    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,
    player: WriteStorage<'a, Player>,

    map: Write<'a, Map>,
    entity: Entities<'a>,
}

pub struct MapgenSystem {
    rng: RandomNumberGenerator,
}

impl<'a> System<'a> for MapgenSystem {
    type SystemData = MapgenSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let start_position = self.gen_map(&mut data);
        self.gen_mobs(start_position, &mut data);
    }
}

impl MapgenSystem {
    pub fn new() -> MapgenSystem {
        MapgenSystem {
            rng: RandomNumberGenerator::new(),
        }
    }

    fn gen_mobs(&mut self, start_position: Position, data: &mut MapgenSystemData) {
        data.entity
            .build_entity()
            .with(start_position, &mut data.position)
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

    fn gen_map(&mut self, data: &mut MapgenSystemData) -> Position {
        let map = &mut data.map;

        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: u16 = 30;
        const MIN_SIZE: u16 = 6;
        const MAX_SIZE: u16 = 10;

        for _ in 0..MAX_ROOMS {
            let w = self.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let h = self.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let x = self.rng.range(0, map.width - w);
            let y = self.rng.range(0, map.height - h);
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                Self::apply_room_to_map(&new_room, map);
                rooms.push(new_room);
            }
        }

        for (a, b) in rooms.iter().zip(rooms.iter().skip(1)) {
            self.connect_rooms(a, b, map);
        }

        rooms[0].center().into()
    }

    fn apply_room_to_map(room: &Rect, map: &mut Map) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                map[&Position::new(x, y)] = TileType::Floor;
            }
        }
    }

    fn connect_rooms(&mut self, a: &Rect, b: &Rect, map: &mut Map) {
        let (prev_x, prev_y) = a.center();
        let (new_x, new_y) = b.center();
        if self.rng.range(0, 2) == 1 {
            Self::apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
            Self::apply_vertical_tunnel(map, prev_y, new_y, new_x);
        } else {
            Self::apply_vertical_tunnel(map, prev_y, new_y, prev_x);
            Self::apply_horizontal_tunnel(map, prev_x, new_x, new_y);
        }
    }

    fn apply_horizontal_tunnel(map: &mut Map, x1: u16, x2: u16, y: u16) {
        for x in min(x1, x2)..=max(x1, x2) {
            let position = &Position::new(x, y);
            if map.contains(position) {
                map[position] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(map: &mut Map, y1: u16, y2: u16, x: u16) {
        for y in min(y1, y2)..=max(y1, y2) {
            let position = &Position::new(x, y);
            if map.contains(position) {
                map[position] = TileType::Floor;
            }
        }
    }
}
