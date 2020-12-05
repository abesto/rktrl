use std::collections::HashSet;

use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::components::position::Position;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "b0f9f696-486d-4771-a7f8-02e8b9f82f3c"]
pub struct Viewshed {
    pub visible_tiles: HashSet<Position>,
    pub revealed_tiles: HashSet<Position>,
    pub range: u16,
    pub dirty: bool,
}
register_serialize!(Viewshed);

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
