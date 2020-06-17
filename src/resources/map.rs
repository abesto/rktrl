use std::cmp::{max, min};
use std::convert::TryInto;
use std::ops::{Index, IndexMut};

use bracket_lib::prelude::*;
use smallvec::SmallVec;
use strum::IntoEnumIterator;

use crate::components::position::Position;
use crate::lib::vector::{Heading, Vector};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub width: i32,
    pub height: i32,
    tiles: Vec<TileType>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height).try_into().unwrap()],
        }
    }

    fn static_xy_idx(width: i32, x: i32, y: i32) -> usize {
        ((y * width) + x).try_into().unwrap()
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        Map::static_xy_idx(self.width, x, y)
    }

    fn static_pos_idx(width: i32, pos: Position) -> usize {
        Map::static_xy_idx(width, pos.x.try_into().unwrap(), pos.y.try_into().unwrap())
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

    fn is_exit_valid(&self, position: Position) -> bool {
        if !self.contains(position) {
            return false;
        }
        self[&position] != TileType::Wall
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

impl Default for Map {
    fn default() -> Self {
        Map::new(80, 50)
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
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let position = self.idx_pos(idx);

        for heading in Heading::iter() {
            let candidate = position + *Vector::unit(heading);
            if self.is_exit_valid(candidate) {
                exits.push((self.pos_idx(candidate), 1.0))
            }
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.idx_pos(idx1);
        let p2 = self.idx_pos(idx2);
        DistanceAlg::Pythagoras.distance2d(*p1, *p2)
    }
}

#[cfg(test)]
mod tests {
    use crate::components::position::Position;
    use crate::resources::map::{Map, MapIterator};

    #[test]
    fn pos_idx_symmetry() {
        let pos = Position::new(10, 15);
        let map = Map::new(30, 49);
        assert_eq!(pos, map.idx_pos(map.pos_idx(pos)));
    }

    #[test]
    fn map_iterator_size_hint() {
        let map = &Map::new(20, 39);

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
