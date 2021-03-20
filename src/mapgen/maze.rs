use crate::systems::prelude::*;
use std::collections::VecDeque;

use super::{common::*, MapBuilder, SnapshotManager};

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell {
            row,
            column,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width - 1 || row > self.height - 1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors: Vec<usize> = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbor_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_column),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1),
            self.calculate_index(current_row, current_column + 1),
        ];

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if !neighbors.is_empty() {
            return if neighbors.len() == 1 {
                Some(neighbors[0])
            } else {
                Some(neighbors[(self.rng.roll_dice(1, neighbors.len() as i32) - 1) as usize])
            };
        }
        None
    }

    fn generate_maze(&mut self, generator: &mut MazeMapBuilder) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);
                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut generator.map);
                generator.take_snapshot();
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        fill(&map.rect(), TileType::Wall, map);

        for cell in self.cells.iter() {
            let x = cell.column * 2;
            let y = cell.row * 2;

            map[(x, y)] = TileType::Floor;
            if !cell.walls[TOP] {
                map[(x, y - 1)] = TileType::Floor;
            }
            if !cell.walls[RIGHT] {
                map[(x + 1, y)] = TileType::Floor;
            }
            if !cell.walls[BOTTOM] {
                map[(x, y + 1)] = TileType::Floor;
            }
            if !cell.walls[LEFT] {
                map[(x - 1, y)] = TileType::Floor;
            }
        }
    }
}

pub struct MazeMapBuilder {
    map: Map,
    snapshot_manager: SnapshotManager,
    // We just pick a random empty spot, so need to store it for it to be stable
    cached_starting_position: Option<Position>,
}

impl MapBuilder for MazeMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        self.build_maze(rng);
    }

    fn spawn_entities(&self, _commands: &mut CommandBuffer, _rng: &mut RandomNumberGenerator) {
        // mazes are empty for now
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.cached_starting_position.unwrap()
    }

    fn get_snapshots(&self) -> VecDeque<Map> {
        self.snapshot_manager.get_snapshots()
    }
}

impl MazeMapBuilder {
    #[must_use]
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        MazeMapBuilder {
            map: Map::new(width, height, depth),
            snapshot_manager: SnapshotManager::new(),
            cached_starting_position: None,
        }
    }

    fn take_snapshot(&mut self) {
        self.snapshot_manager.record_snapshot(self.get_map());
    }

    fn build_maze(&mut self, rng: &mut RandomNumberGenerator) {
        let mut grid = Grid::new(self.map.width / 2, self.map.height / 2, rng);
        grid.generate_maze(self);

        // Pick a random starting position
        let starting_position = random_position_with_tile(TileType::Floor, &self.map, rng);
        self.cached_starting_position = Some(starting_position);

        // Pick a random position for the downwards staircase that's pretty far from the player
        let dijsktra_map = bracket_lib::prelude::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &[self.map.pos_idx(starting_position)],
            &self.map,
            200.0,
        );
        let furthest_reachable_tiles = find_furthest_reachable_tiles(&self.map, &dijsktra_map, 20);
        let stairs_down_pos = rng.random_slice_entry(&furthest_reachable_tiles).unwrap();
        self.map[stairs_down_pos] = TileType::DownStairs;
        self.take_snapshot();
    }
}
