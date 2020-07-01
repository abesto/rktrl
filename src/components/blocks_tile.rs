use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs_derive::Component;

#[derive(PartialEq, Clone, Component, Debug, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct BlocksTile;

impl BlocksTile {
    #[must_use]
    pub const fn new() -> BlocksTile {
        BlocksTile
    }
}

impl Default for BlocksTile {
    fn default() -> Self {
        BlocksTile::new()
    }
}
