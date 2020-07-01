use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

#[derive(Default, Clone, Component, Debug, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Consumable;

#[derive(Component, Clone, Debug, ConvertSaveload)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Copy, Component, Debug, ConvertSaveload)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Clone, Debug, ConvertSaveload)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Clone, Debug, ConvertSaveload)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Copy, Component, Debug, ConvertSaveload)]
pub struct Confusion {
    pub turns: i32,
}
