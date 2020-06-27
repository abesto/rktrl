use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderOrder {
    Items,
    Monsters,
    Player,
}

#[derive(Component)]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: u16,
    pub render_order: RenderOrder,
}
