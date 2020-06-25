use specs::{Component, NullStorage};
use specs_derive::Component;

#[derive(PartialEq, Component, Debug, Clone)]
#[storage(NullStorage)]
pub struct Item;

impl Item {
    pub const fn new() -> Item {
        Item
    }
}

impl Default for Item {
    fn default() -> Self {
        Item::new()
    }
}
