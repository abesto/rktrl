use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Monster;
