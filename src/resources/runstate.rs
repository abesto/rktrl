#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

impl Default for RunState {
    fn default() -> Self {
        RunState::PreRun
    }
}
