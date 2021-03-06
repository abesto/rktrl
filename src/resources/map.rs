use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    convert::TryInto,
    ops::{Index, IndexMut},
    option::*,
};

use bracket_lib::prelude::*;
use legion::Entity;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum::IntoEnumIterator;
use type_uuid::TypeUuid;

use crate::{
    components::position::Position,
    util::vector::{Heading, Vector},
};

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Clone, Serialize, Deserialize, TypeUuid, PartialEq, Debug)]
#[uuid = "a8e9e9cf-6c68-4d18-a3a6-5ce925baa0fb"]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    tiles: Vec<TileType>,
    blocked: HashSet<Position>,
    bloodstains: HashSet<Position>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    tile_content: HashMap<Position, Vec<Entity>>,
}

impl Map {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32) -> Map {
        Map {
            width,
            height,
            depth,
            tiles: vec![TileType::Wall; (width * height).try_into().unwrap()],
            blocked: HashSet::new(),
            bloodstains: HashSet::new(),
            tile_content: HashMap::new(),
        }
    }

    fn static_xy_idx(width: i32, x: i32, y: i32) -> usize {
        ((y * width) + x).try_into().unwrap()
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        Map::static_xy_idx(self.width, x, y)
    }

    fn static_pos_idx(width: i32, pos: Position) -> usize {
        Map::static_xy_idx(width, pos.x, pos.y)
    }

    pub fn pos_idx(&self, pos: Position) -> usize {
        Map::static_pos_idx(self.width, pos)
    }

    pub fn idx_pos(&self, idx: usize) -> Position {
        let idx_i32: i32 = idx.try_into().unwrap();
        let y: i32 = idx_i32 / self.width;
        Position::new(idx_i32 - y * self.width, y)
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn contains(&self, position: Position) -> bool {
        position.x >= 0 && position.y >= 0 && position.x < self.width && position.y < self.height
    }

    pub fn clamp(&self, position: Position) -> Position {
        if self.contains(position) {
            return position;
        }
        Position::new(
            max(0, min(self.width - 1, position.x)),
            max(0, min(self.height - 1, position.y)),
        )
    }

    pub fn is_blocked(&self, position: Position) -> bool {
        if !self.contains(position) {
            return false;
        }
        self.blocked.contains(&position)
    }

    pub fn block(&mut self, position: Position) {
        self.blocked.insert(position);
    }

    fn is_exit_valid(&self, position: Position) -> bool {
        self.contains(position) && !self.is_blocked(position)
    }

    pub fn populate_blocked(&mut self) {
        self.blocked = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_i, tile)| **tile == TileType::Wall)
            .map(|(i, _tile)| self.idx_pos(i))
            .collect();
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.values_mut() {
            content.clear();
        }
    }

    pub fn add_tile_content(&mut self, position: Position, entity: Entity) {
        self.tile_content
            .entry(position)
            .or_insert_with(Vec::new)
            .push(entity);
    }

    pub fn get_tile_contents(&self, position: Position) -> Option<&Vec<Entity>> {
        if !self.tile_content.contains_key(&position) {
            None
        } else {
            Some(&self.tile_content[&position])
        }
    }

    pub fn wall_glyph(
        &self,
        position: Position,
        revealed_tiles: &HashSet<Position>,
    ) -> FontCharType {
        let x = position.x;
        let y = position.y;
        let mut mask: u8 = 0;

        let is_revealed_and_wall = |x, y| {
            let position = Position::new(x, y);
            if !self.contains(position) {
                return false;
            }
            self[&position] == TileType::Wall && revealed_tiles.contains(&position)
        };

        if is_revealed_and_wall(x, y - 1) {
            mask += 1;
        }
        if is_revealed_and_wall(x, y + 1) {
            mask += 2;
        }
        if is_revealed_and_wall(x - 1, y) {
            mask += 4;
        }
        if is_revealed_and_wall(x + 1, y) {
            mask += 8;
        }

        match mask {
            0 => 9,    // Pillar because we can't see neighbors
            1 => 186,  // Wall only to the north
            2 => 186,  // Wall only to the south
            3 => 186,  // Wall to the north and south
            4 => 205,  // Wall only to the west
            5 => 188,  // Wall to the north and west
            6 => 187,  // Wall to the south and west
            7 => 185,  // Wall to the north, south and west
            8 => 205,  // Wall only to the east
            9 => 200,  // Wall to the north and east
            10 => 201, // Wall to the south and east
            11 => 204, // Wall to the north, south and east
            12 => 205, // Wall to the east and west
            13 => 202, // Wall to the east, west, and south
            14 => 203, // Wall to the east, west, and north
            15 => 206, // ╬ Wall on all sides
            _ => 35,   // We missed one?
        }
    }

    pub fn add_bloodstain(&mut self, position: Position) {
        self.bloodstains.insert(position);
    }

    pub fn has_bloodstain(&self, position: Position) -> bool {
        self.bloodstains.contains(&position)
    }

    pub fn get(&self, position: Position) -> Option<TileType> {
        if !self.contains(position) {
            None
        } else {
            Some(self[&position])
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::with_size(0, 0, self.width, self.height)
    }

    pub fn position_set(&self) -> HashSet<Position> {
        self.rect()
            .point_set()
            .iter()
            .map(|p| Position::from(*p))
            .collect()
    }
}

impl Index<&Position> for Map {
    type Output = TileType;

    fn index(&self, pos: &Position) -> &TileType {
        &self.tiles[self.pos_idx(*pos)]
    }
}

impl IndexMut<&Position> for Map {
    fn index_mut(&mut self, pos: &Position) -> &mut TileType {
        &mut self.tiles[Map::static_pos_idx(self.width, *pos)]
    }
}

impl Index<(i32, i32)> for Map {
    type Output = TileType;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self.tiles[Map::static_xy_idx(self.width, index.0, index.1)]
    }
}

impl IndexMut<(i32, i32)> for Map {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut TileType {
        &mut self.tiles[Map::static_xy_idx(self.width, index.0, index.1)]
    }
}

pub struct MapIterator<'a> {
    map: &'a Map,
    next_idx: usize,
    max_idx: usize,
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = (Position, TileType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.max_idx {
            return Option::None;
        }

        let current_pos = self.map.idx_pos(self.next_idx);
        let current_tile = self.map[&current_pos];
        self.next_idx += 1;
        Option::Some((current_pos, current_tile))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.next_idx >= self.map.tile_count() {
            return (0, Option::Some(0));
        }
        let remaining = self.map.tile_count() - self.next_idx;
        (remaining, Option::Some(remaining))
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (Position, TileType);
    type IntoIter = MapIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MapIterator {
            map: self,
            next_idx: 0,
            max_idx: self.tile_count(),
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        idx < self.tiles.len() && self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let position = self.idx_pos(idx);

        Heading::iter()
            .flat_map(|heading| {
                let cardinal = Vector::unit(heading);
                let diagonal = cardinal + cardinal.rotated();
                vec![cardinal, diagonal]
            })
            .map(|vector| position + vector)
            .filter(|candidate| self.is_exit_valid(*candidate))
            .map(|exit| (self.pos_idx(exit), position.distance(exit)))
            .collect()
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.idx_pos(idx1);
        let p2 = self.idx_pos(idx2);
        DistanceAlg::Pythagoras.distance2d(*p1, *p2)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        components::position::Position,
        resources::map::{Map, MapIterator},
    };

    #[test]
    fn pos_idx_symmetry() {
        let pos = Position::new(10, 15);
        let map = Map::new(30, 49, 1);
        assert_eq!(pos, map.idx_pos(map.pos_idx(pos)));
    }

    #[test]
    fn map_iterator_size_hint() {
        let map = &Map::new(20, 39, 1);

        let mut iter: MapIterator = map.into_iter();
        let mut expected: usize = 20 * 39;
        assert_eq!(iter.size_hint().0, expected);
        assert_eq!(iter.size_hint().1.unwrap(), expected);

        while iter.next().is_some() {
            if expected > 0 {
                expected -= 1;
            }
            assert_eq!(iter.size_hint().0, expected);
            assert_eq!(iter.size_hint().1.unwrap(), expected);
        }
    }
}
