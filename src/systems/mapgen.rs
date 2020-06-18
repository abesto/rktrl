use std::cmp::{max, min};

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        blocks_tile::BlocksTile, monster::Monster, name::Name, player::Player, position::Position,
        renderable::Renderable, viewshed::Viewshed,
    },
    lib::rect::Rect,
    resources::map::{Map, TileType},
};

#[derive(SystemData)]
pub struct MapgenSystemData<'a> {
    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,
    player: WriteStorage<'a, Player>,
    viewshed: WriteStorage<'a, Viewshed>,
    monster: WriteStorage<'a, Monster>,
    name: WriteStorage<'a, Name>,
    blocks_tile: WriteStorage<'a, BlocksTile>,

    map: Write<'a, Map>,
    entity: Entities<'a>,
}

pub struct MapgenSystem {
    rng: RandomNumberGenerator,
}

impl<'a> System<'a> for MapgenSystem {
    type SystemData = MapgenSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let rooms = self.gen_map(&mut data);
        self.gen_player(rooms[0].center().into(), &mut data);
        self.gen_monsters(rooms, &mut data);
    }
}

impl MapgenSystem {
    pub fn new() -> MapgenSystem {
        MapgenSystem {
            rng: RandomNumberGenerator::new(),
        }
    }

    fn gen_player(&mut self, start_position: Position, data: &mut MapgenSystemData) {
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
            .with(Viewshed::new(8), &mut data.viewshed)
            .with(BlocksTile::new(), &mut data.blocks_tile)
            .build();
    }

    fn gen_monsters(&mut self, rooms: Vec<Rect>, data: &mut MapgenSystemData) {
        for (i, room) in rooms.iter().skip(1).enumerate() {
            let (x, y) = room.center();
            let (letter, name) = match self.rng.roll_dice(1, 2) {
                1 => ('g', "Goblin"),
                _ => ('o', "Org"),
            };
            data.entity
                .build_entity()
                .with(Position::new(x, y), &mut data.position)
                .with(
                    Renderable {
                        glyph: to_cp437(letter),
                        fg: RGB::named(RED),
                        bg: RGB::named(BLACK),
                    },
                    &mut data.renderable,
                )
                .with(Viewshed::new(8), &mut data.viewshed)
                .with(Monster::new(), &mut data.monster)
                .with(Name::new(format!("{} #{}", name, i)), &mut data.name)
                .with(BlocksTile::new(), &mut data.blocks_tile)
                .build();
        }
    }

    fn gen_map(&mut self, data: &mut MapgenSystemData) -> Vec<Rect> {
        let map = &mut data.map;

        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = self.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let h = self.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let x = self.rng.range(1, map.width - w - 1);
            let y = self.rng.range(1, map.height - h - 1);
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

        rooms
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

    fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let position = Position::new(x, y);
            if map.contains(position) {
                map[&position] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let position = Position::new(x, y);
            if map.contains(position) {
                map[&position] = TileType::Floor;
            }
        }
    }
}
