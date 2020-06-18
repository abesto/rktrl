use specs::{Component, NullStorage};
use specs_derive::Component;

#[derive(PartialEq, Component, Debug, Clone)]
#[storage(NullStorage)]
pub struct Player;

impl Player {
    pub const fn new() -> Player {
        Player
    }
}

impl Default for Player {
    fn default() -> Self {
        Player::new()
    }
}
