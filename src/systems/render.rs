use crate::{
    components::{position::Position, renderable::Renderable},
    resources::map::{Map, TileType},
};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

#[derive(SystemData)]
pub struct RenderSystemData<'a> {
    position: ReadStorage<'a, Position>,
    renderable: ReadStorage<'a, Renderable>,

    map: Read<'a, Map>,
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
        let data = &mut RenderSystemData::fetch(world);
        term.cls();
        self.render_map(data, term);
        self.render_entities(data, term);
    }

    fn render_entities(&mut self, data: &mut RenderSystemData, term: &mut BTerm) {
        for (position, renderable) in (&data.position, &data.renderable).join() {
            term.set(
                position.x.into(),
                position.y.into(),
                renderable.fg,
                renderable.bg,
                renderable.glyph,
            );
        }
    }

    fn render_map(&mut self, data: &mut RenderSystemData, term: &mut BTerm) {
        let map = &data.map;
        for (position, tile) in map.into_iter() {
            // Render a tile depending upon the tile type
            let (fg, bg, c) = match tile {
                TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), '.'),
                TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), '#'),
            };
            term.set(position.x.into(), position.y.into(), fg, bg, to_cp437(c));
        }
    }
}
