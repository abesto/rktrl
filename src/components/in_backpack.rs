use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

#[derive(PartialEq, Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

impl InBackpack {
    #[must_use]
    pub fn new(owner: Entity) -> InBackpack {
        InBackpack { owner }
    }
}
