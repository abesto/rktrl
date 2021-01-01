use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "4f44e6cd-11b0-46fe-9bf9-e4bf42a3a8dd"]
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
