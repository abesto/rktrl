use std::collections::HashSet;
use std::convert::TryFrom;

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, player::Player, position::Position, renderable::Renderable,
        viewshed::Viewshed,
    },
    resources::{
        gamelog::GameLog,
        layout::Layout,
        map::{Map, TileType},
    },
};
use std::convert::TryInto;

#[derive(SystemData)]
pub struct RenderSystemData<'a> {
    position: ReadStorage<'a, Position>,
    renderable: ReadStorage<'a, Renderable>,
    player: ReadStorage<'a, Player>,
    viewshed: ReadStorage<'a, Viewshed>,
    combat_stats: ReadStorage<'a, CombatStats>,

    gamelog: Read<'a, GameLog>,
    layout: ReadExpect<'a, Layout>,
    map: ReadExpect<'a, Map>,
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
        RenderSystem {}
    }

    pub fn run_now_with_term(&mut self, world: &mut World, term: &mut BTerm) {
        let data = &mut RenderSystemData::fetch(world);
        term.cls();
        self.render_map(data, term);
        self.render_entities(data, term);
        self.render_gui(data, term);
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
                position.x,
                position.y,
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
            term.set(position.x, position.y, fg, bg, to_cp437(c));
        }
    }

    fn render_gui(&mut self, data: &mut RenderSystemData, term: &mut BTerm) {
        // This can go away once the fix for
        // https://github.com/thebracket/bracket-lib/issues/96 released
        let bracket_96_workaround = 1;

        // Draw a box around the main bottom gui panel
        let panel_rect = data.layout.panel();
        term.draw_box(
            panel_rect.x1,
            panel_rect.y1,
            panel_rect.width() - bracket_96_workaround,
            panel_rect.height() - bracket_96_workaround,
            RGB::named(WHITE),
            RGB::named(BLACK),
        );

        // Show player health
        let hp_offset: i32 = 12;
        let max_hp_str_len: i32 = 16;
        let hp_bar_offset = hp_offset + max_hp_str_len;

        let stats = (&data.combat_stats, &data.player).join().next().unwrap().0;
        let mut health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        health.truncate(max_hp_str_len.try_into().unwrap());

        term.print_color(
            panel_rect.x1 + hp_offset,
            panel_rect.y1,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            &health,
        );
        term.draw_bar_horizontal(
            panel_rect.x1 + hp_bar_offset,
            panel_rect.y1,
            panel_rect.width() - hp_bar_offset - bracket_96_workaround,
            stats.hp,
            stats.max_hp,
            RGB::named(RED),
            RGB::named(BLACK),
        );

        // Render game log
        data.gamelog
            .entries
            .iter()
            .rev()
            .take(usize::try_from(panel_rect.height()).unwrap() - 2)
            .enumerate()
            .for_each(|(i, message)| {
                term.print(
                    panel_rect.x1 + 2,
                    panel_rect.y1 + 1 + i32::try_from(i).unwrap(),
                    message,
                )
            });
    }
}
