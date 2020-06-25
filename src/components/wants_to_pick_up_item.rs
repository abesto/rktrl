use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug, Clone)]
pub struct WantsToPickUpItem {
    pub item: Entity,
}
