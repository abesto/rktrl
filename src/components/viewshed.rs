use std::collections::HashSet;

use specs::{Component, HashMapStorage};
use specs_derive::Component;

use crate::components::position::Position;

#[derive(PartialEq, Component, Debug, Clone)]
#[storage(HashMapStorage)]
pub struct Viewshed {
    pub visible_tiles: HashSet<Position>,
    pub revealed_tiles: HashSet<Position>,
    pub range: u16,
    pub dirty: bool,
}

impl Viewshed {
    pub fn new(range: u16) -> Viewshed {
        Viewshed {
            visible_tiles: HashSet::new(),
            revealed_tiles: HashSet::new(),
            dirty: true,
            range,
        }
    }
}
