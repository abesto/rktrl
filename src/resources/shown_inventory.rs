use legion::Entity;
use macro_attr::*;
use newtype_derive::*;

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!)]
    pub struct ShownInventory(Vec<Entity>);
}

impl Default for ShownInventory {
    fn default() -> Self {
        vec![].into()
    }
}
