use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum RenderOrder {
    Items,
    Monsters,
    Player,
    Particle,
}

#[derive(Clone)]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: u16,
    pub render_order: RenderOrder,
}
