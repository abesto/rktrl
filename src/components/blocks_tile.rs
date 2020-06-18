use specs::{Component, NullStorage};
use specs_derive::Component;

#[derive(PartialEq, Clone, Component, Debug)]
#[storage(NullStorage)]
pub struct BlocksTile;

impl BlocksTile {
    pub const fn new() -> BlocksTile {
        BlocksTile
    }
}

impl Default for BlocksTile {
    fn default() -> Self {
        BlocksTile::new()
    }
}
