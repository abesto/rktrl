use bracket_lib::prelude::RGB;
use specs::{Component, VecStorage};
use specs_derive::Component;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
}
