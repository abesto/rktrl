use specs::{Component, NullStorage};
use specs_derive::Component;

#[derive(PartialEq, Default, Component, Debug, Clone)]
#[storage(NullStorage)]
pub struct Player;
