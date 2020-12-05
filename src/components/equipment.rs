use legion::Entity;
use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "a43d3b5f-b9a4-45cf-b293-c891fdc0c771"]
pub struct Equippable {
    pub slot: EquipmentSlot,
}
register_serialize!(Equippable);

impl Equippable {
    #[must_use]
    pub fn new(slot: EquipmentSlot) -> Self {
        Equippable { slot }
    }
}

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "2078b3e2-1f22-4047-ba28-fe43b2786fd8"]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}
register_serialize!(Equipped);
