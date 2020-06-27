use specs::prelude::Entity;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
}

impl RunState {
    #[must_use]
    pub fn show_inventory(self) -> bool {
        self == RunState::ShowDropItem || self == RunState::ShowInventory
    }
}

impl Default for RunState {
    fn default() -> Self {
        RunState::PreRun
    }
}
