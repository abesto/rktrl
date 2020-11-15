use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesFood;
