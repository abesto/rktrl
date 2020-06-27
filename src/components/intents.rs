use crate::components::position::Position;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug, Clone)]
pub struct MeleeIntent {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct PickupIntent {
    pub item: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct UseIntent {
    pub item: Entity,
    pub target: Option<Position>,
}

#[derive(Component, Debug, Clone)]
pub struct DropIntent {
    pub item: Entity,
}
