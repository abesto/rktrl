use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Consumable;

#[derive(Clone, Debug)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Confusion {
    pub turns: i32,
}
