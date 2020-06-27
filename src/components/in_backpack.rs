use specs::prelude::*;
use specs_derive::Component;

#[derive(PartialEq, Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

impl InBackpack {
    #[must_use]
    pub fn new(owner: Entity) -> InBackpack {
        InBackpack { owner }
    }
}
