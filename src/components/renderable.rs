use bracket_lib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: u16,
}
