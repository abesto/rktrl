use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::components::position::Position;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Viewshed {
    pub visible_tiles: HashSet<Position>,
    pub revealed_tiles: HashSet<Position>,
    pub range: u16,
    pub dirty: bool,
}

impl Viewshed {
    #[must_use]
    pub fn new(range: u16) -> Viewshed {
        Viewshed {
            visible_tiles: HashSet::new(),
            revealed_tiles: HashSet::new(),
            dirty: true,
            range,
        }
    }
}
