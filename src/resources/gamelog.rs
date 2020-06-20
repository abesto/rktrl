pub struct GameLog {
    pub entries: Vec<String>,
}

impl Default for GameLog {
    fn default() -> Self {
        GameLog { entries: vec![] }
    }
}
