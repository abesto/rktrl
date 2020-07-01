use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage};
use specs_derive::Component;

#[derive(PartialEq, Default, Component, Debug, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Player;
