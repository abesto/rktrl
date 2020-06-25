#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
}

impl Default for RunState {
    fn default() -> Self {
        RunState::PreRun
    }
}
