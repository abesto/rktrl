use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

impl Equippable {
    #[must_use]
    pub fn new(slot: EquipmentSlot) -> Self {
        Equippable { slot }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}
