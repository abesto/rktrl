use macro_attr::*;
use newtype_derive::*;
use shipyard::EntityId;

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!)]
    pub struct ShownInventory(Vec<EntityId>);
}

impl Default for ShownInventory {
    fn default() -> Self {
        vec![].into()
    }
}
