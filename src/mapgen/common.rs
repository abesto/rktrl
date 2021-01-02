use crate::systems::prelude::*;
use std::cmp::{max, min};

pub fn apply_room_to_map(room: &Rect, map: &mut Map) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[&Position::new(x, y)] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let position = Position::new(x, y);
        if map.contains(position) {
            map[&position] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let position = Position::new(x, y);
        if map.contains(position) {
            map[&position] = TileType::Floor;
        }
    }
}

pub fn connect_rooms(a: &Rect, b: &Rect, map: &mut Map, rng: &mut RandomNumberGenerator) {
    let prev = a.center();
    let new = b.center();
    if rng.range(0, 2) == 1 {
        apply_horizontal_tunnel(map, prev.x, new.x, prev.y);
        apply_vertical_tunnel(map, prev.y, new.y, new.x);
    } else {
        apply_vertical_tunnel(map, prev.y, new.y, prev.x);
        apply_horizontal_tunnel(map, prev.x, new.x, new.y);
    }
}
