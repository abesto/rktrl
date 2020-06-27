#[derive(PartialEq, Clone, Debug)]
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
