use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::ConvertSaveload;

#[derive(Clone, ConvertSaveload)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl Default for GameLog {
    fn default() -> Self {
        GameLog { entries: vec![] }
    }
}
