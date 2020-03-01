use specs::{Component, HashMapStorage};
use specs_derive::Component;

#[derive(PartialEq, Component, Debug, Clone)]
#[storage(HashMapStorage)]
pub struct Player;

impl Player {
    pub fn new() -> Player {
        Player
    }
}
