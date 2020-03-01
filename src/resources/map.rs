use crate::components::position::Position;
use std::ops::{Index, IndexMut};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<TileType>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Floor; width * height],
        }
    }

    fn static_xy_idx(width: usize, x: i32, y: i32) -> usize {
        (y as usize * width) + x as usize
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        Map::static_xy_idx(self.width, x, y)
    }

    fn static_pos_idx(width: usize, pos: &Position) -> usize {
        Map::static_xy_idx(width, pos.x, pos.y)
    }

    pub fn pos_idx(&self, pos: &Position) -> usize {
        Map::static_pos_idx(self.width, pos)
    }

    pub fn idx_pos(&self, idx: i32) -> Position {
        let y = idx / self.width as i32;
        Position {
            x: idx - y * self.width as i32,
            y,
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

impl Index<&Position> for Map {
    type Output = TileType;

    fn index(&self, pos: &Position) -> &TileType {
        &self.tiles[self.pos_idx(pos)]
    }
}

impl IndexMut<&Position> for Map {
    fn index_mut(&mut self, pos: &Position) -> &mut TileType {
        &mut self.tiles[Map::static_pos_idx(self.width, pos)]
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
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = (Position, TileType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx == self.map.width * self.map.height {
            return Option::None;
        }

        let current_pos = self.map.idx_pos(self.next_idx as i32);
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
        }
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
        assert_eq!(pos, map.idx_pos(map.pos_idx(&pos) as i32));
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
