use crate::systems::prelude::*;

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

use crate::util::rect_ext::RectExt;
use itertools::Itertools;
use rand::distributions::uniform::UniformSampler;
use smallvec::alloc::collections::VecDeque;

pub fn apply_room_to_map(room: &Rect, map: &mut Map) {
    fill(room, TileType::Floor, map);
}

pub fn fill(room: &Rect, tile: TileType, map: &mut Map) {
    for p in room.point_set() {
        map[&Position::from(p)] = tile;
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

pub fn connect_positions(a: Position, b: Position, map: &mut Map, rng: &mut RandomNumberGenerator) {
    if rng.range(0, 2) == 1 {
        apply_horizontal_tunnel(map, a.x, b.x, a.y);
        apply_vertical_tunnel(map, a.y, b.y, b.x);
    } else {
        apply_vertical_tunnel(map, a.y, b.y, a.x);
        apply_horizontal_tunnel(map, a.x, b.x, b.y);
    }
}

pub fn connect_rooms(a: &Rect, b: &Rect, map: &mut Map, rng: &mut RandomNumberGenerator) {
    connect_positions(a.center().into(), b.center().into(), map, rng);
}

fn position_if_in_map(position: Position, map: &Map) -> Option<Position> {
    if map.contains(position) {
        Some(position)
    } else {
        None
    }
}

pub fn walk(from: Position, direction: Heading, map: &Map) -> Vec<Position> {
    let step = Vector::unit(direction);
    itertools::unfold(from, |pos| {
        *pos += step;
        position_if_in_map(*pos, map)
    })
    .collect()
}

/// Cast a ray `from` a position in a `direction`, returning the position of the first `look_for`
/// tile that is found, if any
pub fn raycast(
    from: Position,
    direction: Heading,
    look_for: TileType,
    map: &Map,
) -> Option<Position> {
    walk(from, direction, map)
        .iter()
        .find(|&pos| map[pos] == look_for)
        .cloned()
}

/// Cast `n` rays in `ray_direction`, starting at `from`, walking in `walk_direction`,
/// returning all positions where a `look_for` tile was found.
pub fn raycast_walk(
    from: Position,
    ray_direction: Heading,
    walk_direction: Heading,
    n: usize,
    look_for: TileType,
    map: &Map,
) -> Vec<Position> {
    walk(from, walk_direction, map)
        .iter()
        .take(n)
        .flat_map(|&ray_from| raycast(ray_from, ray_direction, look_for, map))
        .collect()
}

/// Return a mapping from "which side is this" to "positions along the hull"
pub fn raycast_hull(bbox: Rect, look_for: TileType, map: &Map) -> HashMap<Heading, Vec<Position>> {
    let mut retval = HashMap::new();

    let n = |walk_direction: Heading| match walk_direction {
        Heading::North | Heading::South => (bbox.height() + 1).try_into().unwrap(),
        Heading::East | Heading::West => (bbox.width() + 1).try_into().unwrap(),
    };

    let mut record_raycast_walk = |from: Position, walk_direction: Heading| {
        retval.insert(
            walk_direction.rotate_ccw(),
            raycast_walk(
                from,
                walk_direction.rotate_cw(),
                walk_direction,
                n(walk_direction),
                look_for,
                map,
            ),
        );
    };

    // Raycast inward, walking along each edge of `bbox`
    record_raycast_walk(bbox.top_left(), Heading::East);
    record_raycast_walk(bbox.top_right(), Heading::South);
    record_raycast_walk(bbox.bottom_right(), Heading::West);
    record_raycast_walk(bbox.bottom_left(), Heading::North);

    // Restrict matches to bbox
    for values in retval.values_mut() {
        values.retain(|position| bbox.point_in_rect(**position))
    }

    retval
}

/// Add exactly one corridor to connect the TileSet::Floor subsets of `a` and `b`.
/// Guarantees a fully connected area IFF both `a` and `b` are themselves internally
/// fully connected.
pub fn connect_regions(a: Rect, b: Rect, map: &mut Map, rng: &mut RandomNumberGenerator) {
    let a_hull = raycast_hull(a, TileType::Floor, map);
    let b_hull = raycast_hull(b, TileType::Floor, map);

    // Direct vertical?
    {
        let (north_hull, south_hull) = if a.y2 <= b.y1 {
            (&a_hull, &b_hull)
        } else {
            (&b_hull, &a_hull)
        };
        let north_x = north_hull[&Heading::South]
            .iter()
            .map(|p| p.x)
            .collect::<HashSet<_>>();
        let south_x = south_hull[&Heading::North]
            .iter()
            .map(|p| p.x)
            .collect::<HashSet<_>>();
        let facing_x = north_x.intersection(&south_x).collect_vec();
        if !facing_x.is_empty() {
            let &&x = rng.random_slice_entry(&facing_x).unwrap();
            let north_position = north_hull[&Heading::South]
                .iter()
                .find(|p| p.x == x)
                .unwrap();
            let south_position = south_hull[&Heading::North]
                .iter()
                .find(|p| p.x == x)
                .unwrap();
            connect_positions(*north_position, *south_position, map, rng);
            return;
        }
    }

    // Direct horizontal?
    {
        let (west_hull, east_hull) = if a.x2 <= b.x1 {
            (&a_hull, &b_hull)
        } else {
            (&b_hull, &a_hull)
        };
        let west_y = west_hull[&Heading::East]
            .iter()
            .map(|p| p.y)
            .collect::<HashSet<_>>();
        let east_y = east_hull[&Heading::West]
            .iter()
            .map(|p| p.y)
            .collect::<HashSet<_>>();
        let facing_y = west_y.intersection(&east_y).collect_vec();
        if !facing_y.is_empty() {
            let &&y = rng.random_slice_entry(&facing_y).unwrap();
            let west_position = west_hull[&Heading::East].iter().find(|p| p.y == y).unwrap();
            let east_position = east_hull[&Heading::West].iter().find(|p| p.y == y).unwrap();
            connect_positions(*west_position, *east_position, map, rng);
            return;
        }
    }

    // No direct vertical / horizontal corridor is possible, do a corner
    {
        let mut a_faces = vec![];
        let mut b_faces = vec![];

        // Figure out valid candidates for which sides we'll connect
        if a.x2 <= b.x1 {
            a_faces.push(Heading::East);
            b_faces.push(Heading::West);
        } else {
            a_faces.push(Heading::West);
            b_faces.push(Heading::East);
        }
        if a.y2 <= b.y1 {
            a_faces.push(Heading::South);
            b_faces.push(Heading::North);
        } else {
            a_faces.push(Heading::North);
            b_faces.push(Heading::South);
        }

        // Randomly determine which way the corner goes
        let (a_face, b_face) = {
            if rng.roll_dice(1, 2) == 1 {
                (a_faces[0], b_faces[1])
            } else {
                (a_faces[1], b_faces[0])
            }
        };

        // Pick two random points along those faces
        let a_position = rng.random_slice_entry(&a_hull[&a_face]).unwrap();
        let b_position = rng.random_slice_entry(&b_hull[&b_face]).unwrap();

        // And finally, do the thing!
        connect_positions(*a_position, *b_position, map, rng);
    }
}

pub fn walls_around(rect: &Rect, map: &mut Map) {
    for x in rect.x1..=rect.x2 {
        map[(x, rect.y1)] = TileType::Wall;
        map[(x, rect.y2)] = TileType::Wall;
    }
    for y in rect.y1..=rect.y2 {
        map[(rect.x1, y)] = TileType::Wall;
        map[(rect.x2, y)] = TileType::Wall;
    }
}

pub fn connected_region(seed: Position, map: &Map) -> Vec<Position> {
    let target = map[&seed];
    let mut retval = vec![seed];

    let mut queue = VecDeque::new();
    queue.push_back(seed);

    while !queue.is_empty() {
        let p = queue.pop_front().unwrap();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let p_next = p + Vector::new(dx, dy);
                if map.get(p_next) == Some(target) && !retval.contains(&p_next) {
                    retval.push(p_next);
                    queue.push_back(p_next);
                }
            }
        }
    }

    retval
}

pub fn random_position_with_tile(
    target: TileType,
    map: &Map,
    rng: &mut RandomNumberGenerator,
) -> Position {
    let position_sampler =
        PositionSampler::new(Position::new(0, 0), Position::from(map.dimensions()));
    loop {
        let p = position_sampler.sample(rng.get_rng());
        println!("{:?} -> {:?}", p, map[&p]);
        if map[&p] == target {
            break p;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mapgen::common::*;

    fn make_map() -> Map {
        /*
         01234567890123456789
        0####################
        1####################
        2####################
        3###############...##
        4###############...##
        5#####......####...##
        6#####.............##
        7#####......####...##
        8#####......####...##
        9#####......####...##
        0#####......####...##
        1####################
        2####################
        3####################
        4####################
        5##.................#
        6##.................#
        7##.................#
        8##.................#
        9####################
        */
        let mut map = Map::new(20, 20, 1);
        apply_room_to_map(&Rect::with_exact(5, 5, 11, 11), &mut map);
        apply_room_to_map(&Rect::with_exact(15, 3, 18, 11), &mut map);
        apply_room_to_map(&Rect::with_exact(2, 17, 19, 19), &mut map);
        apply_horizontal_tunnel(&mut map, 10, 15, 6);
        map
    }

    #[test]
    fn test_raycast() {
        let map = make_map();
        assert_eq!(
            raycast(Position::new(6, 0), Heading::South, TileType::Floor, &map),
            Some(Position::new(6, 5))
        );
        assert_eq!(
            raycast(Position::new(18, 7), Heading::West, TileType::Floor, &map),
            Some(Position::new(17, 7))
        );
        assert_eq!(
            raycast(Position::new(0, 0), Heading::South, TileType::Floor, &map),
            None
        );
    }

    #[test]
    fn test_raycast_hull() {
        let map = make_map();
        let hull = raycast_hull(Rect::with_exact(0, 0, 19, 12), TileType::Floor, &map);

        assert_eq!(
            hull[&Heading::North],
            vec!(
                // Room 1
                Position::new(5, 5),
                Position::new(6, 5),
                Position::new(7, 5),
                Position::new(8, 5),
                Position::new(9, 5),
                Position::new(10, 5),
                // Corridor
                Position::new(11, 6),
                Position::new(12, 6),
                Position::new(13, 6),
                Position::new(14, 6),
                // Room 2
                Position::new(15, 3),
                Position::new(16, 3),
                Position::new(17, 3),
            )
        );
        assert_eq!(
            hull[&Heading::East],
            vec!(
                Position::new(17, 3),
                Position::new(17, 4),
                Position::new(17, 5),
                Position::new(17, 6),
                Position::new(17, 7),
                Position::new(17, 8),
                Position::new(17, 9),
                Position::new(17, 10),
            )
        );
        assert_eq!(
            hull[&Heading::South],
            vec!(
                // Room 2
                Position::new(17, 10),
                Position::new(16, 10),
                Position::new(15, 10),
                // Corridor
                Position::new(14, 6),
                Position::new(13, 6),
                Position::new(12, 6),
                Position::new(11, 6),
                // Room 1
                Position::new(10, 10),
                Position::new(9, 10),
                Position::new(8, 10),
                Position::new(7, 10),
                Position::new(6, 10),
                Position::new(5, 10),
            )
        );
        assert_eq!(
            hull[&Heading::West],
            vec!(
                Position::new(5, 10),
                Position::new(5, 9),
                Position::new(5, 8),
                Position::new(5, 7),
                Position::new(5, 6),
                Position::new(5, 5),
                Position::new(15, 4),
                Position::new(15, 3),
            )
        );
    }

    #[test]
    fn test_connect_regions_direct_vertical() {
        let mut map = make_map();
        connect_regions(
            Rect::with_size(4, 2, 14, 9),
            Rect::with_size(0, 14, 20, 5),
            &mut map,
            &mut RandomNumberGenerator::new(),
        );

        let new_hull = raycast_hull(Rect::with_size(0, 0, 20, 20), TileType::Floor, &map);
        assert!(new_hull[&Heading::East]
            .iter()
            .filter(|p| p.y >= 11 && p.y <= 14)
            .all(|p| p.x >= 5 && p.x <= 17));
    }

    #[test]
    fn test_connected_regions() {
        let map = make_map();
        assert_eq!(connected_region(Position::new(7, 7), &map).len(), 18);
    }
}
