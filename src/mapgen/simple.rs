use super::{common::*, MapBuilder};
use crate::systems::prelude::*;

pub struct SimpleMapBuilder {
    rooms: Vec<Rect>,
    map: Map,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        self.rooms = SimpleMapBuilder::rooms_and_corridors(&mut self.map, rng);
    }

    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator) {
        for room in &self.rooms {
            super::spawner::room(rng, room, self.map.depth, commands);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.rooms[0].center().into()
    }
}

impl SimpleMapBuilder {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        SimpleMapBuilder {
            rooms: vec![],
            map: Map::new(width, height, depth),
        }
    }

    fn rooms_and_corridors(map: &mut Map, rng: &mut RandomNumberGenerator) -> Vec<Rect> {
        let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE + 1);
            let h = rng.range(MIN_SIZE, MAX_SIZE + 1);
            let x = rng.range(1, map.width - w - 1);
            let y = rng.range(1, map.height - h - 1);
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                apply_room_to_map(&new_room, map);
                rooms.push(new_room);
            }
        }

        for (a, b) in rooms.iter().zip(rooms.iter().skip(1)) {
            connect_rooms(a, b, map, rng);
        }

        // Place stairs down in last room
        let last_room = rooms.last().unwrap();
        map[&last_room.center().into()] = TileType::DownStairs;

        rooms
    }
}
