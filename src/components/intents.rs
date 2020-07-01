use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

use crate::components::position::Position;

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct MeleeIntent {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct PickupIntent {
    pub item: Entity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseTarget {
    SelfCast,
    Position(Position),
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct UseIntent {
    pub item: Entity,
    pub target: UseTarget,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct DropIntent {
    pub item: Entity,
}
