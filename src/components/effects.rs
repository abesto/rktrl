use specs::prelude::*;
use specs_derive::Component;

#[derive(Default, Component, Debug)]
#[storage(NullStorage)]
pub struct Consumable;

#[derive(PartialEq, Component, Debug, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

impl ProvidesHealing {
    #[must_use]
    pub const fn new(heal_amount: i32) -> ProvidesHealing {
        ProvidesHealing { heal_amount }
    }
}
