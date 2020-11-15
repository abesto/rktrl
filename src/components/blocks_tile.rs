use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
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
