use legion::Entity;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

impl Equippable {
    #[must_use]
    pub fn new(slot: EquipmentSlot) -> Self {
        Equippable { slot }
    }
}

#[derive(Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}
