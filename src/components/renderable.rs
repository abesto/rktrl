use bracket_lib::prelude::*;
use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum RenderOrder {
    Items,
    Monsters,
    Player,
    Particle,
}

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "a5b0f482-a553-441b-a600-6f4985fa0f5d"]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: u16,
    pub render_order: RenderOrder,
}

register_serialize!(Renderable);
