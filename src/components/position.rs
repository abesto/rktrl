use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};
use specs_derive::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}