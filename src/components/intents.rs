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
}
