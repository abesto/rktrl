use crate::systems::prelude::*;
use std::collections::VecDeque;

use super::{common::*, MapBuilder, SnapshotManager};

pub struct SimpleMapBuilder {
    rooms: Vec<Rect>,
    map: Map,
    snapshot_manager: SnapshotManager,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        self.rooms_and_corridors(rng);
    }

    fn spawn_entities(&self, commands: &mut CommandBuffer, rng: &mut RandomNumberGenerator) {
        for room in self.rooms.iter().skip(1) {
            super::spawner::spawn_room(rng, room, self.map.depth, commands);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.rooms[0].center().into()
    }

    fn get_snapshots(&self) -> VecDeque<Map> {
        self.snapshot_manager.get_snapshots()
    }
}

impl SimpleMapBuilder {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        SimpleMapBuilder {
            rooms: vec![],
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
        }
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }

    fn rooms_and_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        self.take_snapshot();

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE + 1);
            let h = rng.range(MIN_SIZE, MAX_SIZE + 1);
            let x = rng.range(1, self.map.width - w - 1);
            let y = rng.range(1, self.map.height - h - 1);
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                apply_room_to_map(&new_room, &mut self.map);
                self.take_snapshot();
                self.rooms.push(new_room);
            }
        }

        for i in 0..self.rooms.len() - 1 {
            connect_rooms(&self.rooms[i], &self.rooms[i + 1], &mut self.map, rng);
            self.take_snapshot();
        }

        // Place stairs down in last room
        let last_room = self.rooms.last().unwrap();
        self.map[&last_room.center().into()] = TileType::DownStairs;
    }
}
