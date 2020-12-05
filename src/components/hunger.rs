use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "b3611d79-61cc-485c-876a-dd6c3d307219"]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}
register_serialize!(HungerClock);

impl HungerClock {
    #[must_use]
    pub const fn new(state: HungerState, duration: i32) -> Self {
        HungerClock { state, duration }
    }
}

impl Default for HungerClock {
    fn default() -> Self {
        HungerClock::new(HungerState::WellFed, 20)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeUuid)]
#[uuid = "0ce5c8a0-3b75-415c-a7d3-99605a9b09ca"]
pub struct ProvidesFood;
register_serialize!(ProvidesFood);
