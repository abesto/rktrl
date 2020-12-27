use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Default, Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "bbd7b852-d0ec-49ed-a614-14b73d7f08b2"]
pub struct Consumable;
register_serialize!(Consumable);

#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "e128019a-1530-439d-ba25-fb19767597a9"]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}
register_serialize!(ProvidesHealing);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "6698e9da-3ea3-4060-83b8-93b6e268b5fd"]
pub struct Ranged {
    pub range: i32,
}
register_serialize!(Ranged);

#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "376f82dc-b711-411b-ba70-b7f187362e6e"]
pub struct InflictsDamage {
    pub damage: i32,
}
register_serialize!(InflictsDamage);

#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "941017b1-03ae-4e8c-b8c9-a4c44e4cdb94"]
pub struct AreaOfEffect {
    pub radius: i32,
}
register_serialize!(AreaOfEffect);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "e8d51bb0-5946-426e-affc-beca5788200f"]
pub struct Confusion {
    pub turns: i32,
}
register_serialize!(Confusion);

impl Confusion {
    pub fn tick(&self) -> Option<Confusion> {
        if self.turns <= 0 {
            return None;
        }
        Some(Confusion {
            turns: self.turns - 1,
        })
    }
}
