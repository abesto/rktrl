use specs::prelude::*;
use specs_derive::Component;

use crate::components::position::Position;

#[derive(Component, Debug, Clone)]
pub struct MeleeIntent {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct PickupIntent {
    pub item: Entity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseTarget {
    SelfCast,
    Position(Position),
}

#[derive(Component, Debug, Clone)]
pub struct UseIntent {
    pub item: Entity,
    pub target: UseTarget,
}

#[derive(Component, Debug, Clone)]
pub struct DropIntent {
    pub item: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct RemoveIntent {
    pub item: Entity,
}
