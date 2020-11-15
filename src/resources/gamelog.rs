#[derive(Clone)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl Default for GameLog {
    fn default() -> Self {
        GameLog { entries: vec![] }
    }
}

impl GameLog {
    pub fn push<S: ToString>(&mut self, msg: S) {
        self.entries.push(msg.to_string());
    }
}
