use std::cmp::{max, min};

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use specs::shrev::EventChannel;

use crate::{
    components::position::Position,
    resources::map::{Map, TileType},
    systems::spawner::SpawnRequest,
};

#[derive(SystemData)]
pub struct MapgenSystemData<'a> {
    rng: WriteExpect<'a, RandomNumberGenerator>,
    map: WriteExpect<'a, Map>,
    spawn_requests: WriteExpect<'a, EventChannel<SpawnRequest>>,
}

pub struct MapgenSystem;

impl<'a> System<'a> for MapgenSystem {
    type SystemData = MapgenSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let rooms = self.gen_map(&mut data);

        // Request player spawn
        data.spawn_requests
            .single_write(SpawnRequest::Player(rooms[0].center().into()));

        // Request monster spawns
        let depth = data.map.depth;
        data.spawn_requests.iter_write(
            rooms
                .iter()
                .skip(1)
                .map(|&rect| SpawnRequest::Room { rect, depth }),
        );
    }
}

impl MapgenSystem {
    fn gen_map(&mut self, data: &mut MapgenSystemData) -> Vec<Rect> {
        let map = &mut data.map;

        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = data.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let h = data.rng.range(MIN_SIZE, MAX_SIZE + 1);
            let x = data.rng.range(1, map.width - w - 1);
            let y = data.rng.range(1, map.height - h - 1);
            let new_room = Rect::with_size(x, y, w, h);
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
            self.connect_rooms(a, b, map, &mut data.rng);
        }

        // Place stairs down in last room
        let last_room = rooms.last().unwrap();
        data.map[&last_room.center().into()] = TileType::DownStairs;

        rooms
    }

    fn apply_room_to_map(room: &Rect, map: &mut Map) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                map[&Position::new(x, y)] = TileType::Floor;
            }
        }
    }

    fn connect_rooms(
        &mut self,
        a: &Rect,
        b: &Rect,
        map: &mut Map,
        rng: &mut RandomNumberGenerator,
    ) {
        let prev = a.center();
        let new = b.center();
        if rng.range(0, 2) == 1 {
            Self::apply_horizontal_tunnel(map, prev.x, new.x, prev.y);
            Self::apply_vertical_tunnel(map, prev.y, new.y, new.x);
        } else {
            Self::apply_vertical_tunnel(map, prev.y, new.y, prev.x);
            Self::apply_horizontal_tunnel(map, prev.x, new.x, new.y);
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
