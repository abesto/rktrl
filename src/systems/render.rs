use crate::{
    components::{player::Player, position::Position, renderable::Renderable, viewshed::Viewshed},
    resources::map::{Map, TileType},
};
use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use std::collections::HashSet;

#[derive(SystemData)]
pub struct RenderSystemData<'a> {
    position: ReadStorage<'a, Position>,
    renderable: ReadStorage<'a, Renderable>,
    player: ReadStorage<'a, Player>,
    viewshed: ReadStorage<'a, Viewshed>,

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

    fn player_visible_tiles(&mut self, data: &mut RenderSystemData) -> HashSet<Position> {
        (&data.player, &data.viewshed)
            .join()
            .flat_map(|t| t.1.visible_tiles.clone())
            .collect()
    }

    fn player_revealed_tiles(&mut self, data: &mut RenderSystemData) -> HashSet<Position> {
        (&data.player, &data.viewshed)
            .join()
            .flat_map(|t| t.1.revealed_tiles.clone())
            .collect()
    }

    fn render_entities(&mut self, data: &mut RenderSystemData, term: &mut BTerm) {
        let visible = self.player_visible_tiles(data);
        for (position, renderable) in (&data.position, &data.renderable).join() {
            if !visible.contains(position) {
                continue;
            }
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
        let visible = self.player_visible_tiles(data);
        let revealed = self.player_revealed_tiles(data);

        for position in revealed {
            let tile = data.map[&position];
            let (mut fg, bg, c) = match tile {
                TileType::Floor => (RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), '.'),
                TileType::Wall => (RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), '#'),
            };
            if !visible.contains(&position) {
                fg = fg.to_greyscale();
            }
            term.set(position.x.into(), position.y.into(), fg, bg, to_cp437(c));
        }
    }
}
