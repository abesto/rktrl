use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum RenderOrder {
    Items,
    Monsters,
    Player,
}

#[derive(Component, Clone, ConvertSaveload)]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: u16,
    pub render_order: RenderOrder,
}
