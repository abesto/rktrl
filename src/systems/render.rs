use crate::components::{position::Position, renderable::Renderable};
use bracket_lib::prelude::{BTerm, Console};
use shred_derive::SystemData;
use specs::prelude::*;

#[derive(SystemData)]
pub struct RenderSystemData<'a> {
    position: ReadStorage<'a, Position>,
    renderable: ReadStorage<'a, Renderable>,
}

pub struct RenderSystem {}

impl<'a> System<'a> for RenderSystem {
    type SystemData = RenderSystemData<'a>;

    fn run(&mut self, _data: Self::SystemData) {
        unimplemented!();
        // bracket-lib requires the BTerm to be *moved* into main_loop(),
        // so we need to borrow it on each tick.
        // The only way I know of doing that is by implementing
        // run_now_with_term below, breaking the normal System interface.
        // Logic should move back here if/when specs-bracket-lib integration is improved.
    }
}

impl<'a> RenderSystem {
    pub fn new() -> RenderSystem {
        return RenderSystem {};
    }

    pub fn run_now_with_term(&mut self, world: &mut World, term: &mut BTerm) {
        let data = RenderSystemData::fetch(world);
        term.cls();
        for (position, renderable) in (&data.position, &data.renderable).join() {
            term.set(
                position.x,
                position.y,
                renderable.fg,
                renderable.bg,
                renderable.glyph,
            );
        }
    }
}
