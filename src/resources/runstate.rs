#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

impl Default for RunState {
    fn default() -> Self {
        RunState::Paused
    }
}
