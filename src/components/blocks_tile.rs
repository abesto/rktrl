use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "ccadcaae-e8cc-4e75-b7bc-26cea7a6c286"]
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
