use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "ae603a9d-07a5-47e7-8000-e8c27de416e7"]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}
register_serialize!(CombatStats);

impl CombatStats {
    pub fn with_hp(&self, new_hp: i32) -> CombatStats {
        let mut new = self.clone();
        new.hp = new_hp;
        new
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "8863a392-1ded-427f-9c9f-9b02fc87dfd8"]
pub struct MeleePowerBonus {
    pub power: i32,
}
register_serialize!(MeleePowerBonus);

impl MeleePowerBonus {
    #[must_use]
    pub fn new(power: i32) -> Self {
        MeleePowerBonus { power }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "5e42c635-43db-4061-b6fa-9563a99c1f06"]
pub struct DefenseBonus {
    pub defense: i32,
}
register_serialize!(DefenseBonus);

impl DefenseBonus {
    #[must_use]
    pub fn new(defense: i32) -> Self {
        DefenseBonus { defense }
    }
}
