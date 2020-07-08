use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

#[derive(Component, Debug, ConvertSaveload)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleePowerBonus {
    pub power: i32,
}

impl MeleePowerBonus {
    #[must_use]
    pub fn new(power: i32) -> Self {
        MeleePowerBonus { power }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct DefenseBonus {
    pub defense: i32,
}

impl DefenseBonus {
    #[must_use]
    pub fn new(defense: i32) -> Self {
        DefenseBonus { defense }
    }
}
