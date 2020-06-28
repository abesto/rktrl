use specs::prelude::*;
use specs_derive::Component;

#[derive(Default, Component, Debug)]
#[storage(NullStorage)]
pub struct Consumable;

#[derive(Component, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Copy, Component, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}
