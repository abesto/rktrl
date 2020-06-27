use std::collections::HashSet;
use std::convert::TryFrom;

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, in_backpack::InBackpack, name::Name, player::Player,
        position::Position, renderable::Renderable, viewshed::Viewshed,
    },
    lib::{rect_ext::RectExt, vector::Vector},
    resources::{
        gamelog::GameLog,
        layout::Layout,
        map::{Map, TileType},
        mouse_pos::MousePos,
        runstate::RunState,
        shown_inventory::ShownInventory,
    },
};
use std::convert::TryInto;

#[derive(SystemData)]
pub struct RenderSystemData<'a> {
    entity: Entities<'a>,
    position: ReadStorage<'a, Position>,
    renderable: ReadStorage<'a, Renderable>,
    player: ReadStorage<'a, Player>,
    viewshed: ReadStorage<'a, Viewshed>,
    combat_stats: ReadStorage<'a, CombatStats>,
    name: ReadStorage<'a, Name>,
    backpack: ReadStorage<'a, InBackpack>,

    gamelog: Read<'a, GameLog>,
    layout: ReadExpect<'a, Layout>,
    map: ReadExpect<'a, Map>,
    runstate: Read<'a, RunState>,
    mouse_pos: Read<'a, MousePos>,
    shown_inventory: Write<'a, ShownInventory>,
}

pub struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = RenderSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let draw_batch = &mut DrawBatch::new();
        draw_batch.cls();
        self.render_map(&mut data, draw_batch);
        self.render_entities(&mut data, draw_batch);
        self.render_gui(&mut data, draw_batch);
        self.draw_tooltips(&mut data, draw_batch);
        self.show_inventory(&mut data, draw_batch);
        draw_batch.submit(0).unwrap();
    }
}

impl<'a> RenderSystem {
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

    fn render_entities(&mut self, data: &mut RenderSystemData, draw_batch: &mut DrawBatch) {
        let visible = self.player_visible_tiles(data);
        for (position, renderable) in (&data.position, &data.renderable).join() {
            if !visible.contains(position) {
                continue;
            }
            draw_batch.set(**position, renderable.color, renderable.glyph);
        }
    }

    fn render_map(&mut self, data: &mut RenderSystemData, draw_batch: &mut DrawBatch) {
        let visible = self.player_visible_tiles(data);
        let revealed = self.player_revealed_tiles(data);

        for position in revealed {
            let tile = data.map[&position];
            let (mut color, c) = match tile {
                TileType::Floor => (
                    ColorPair::new(RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.)),
                    '.',
                ),
                TileType::Wall => (
                    ColorPair::new(RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.)),
                    '#',
                ),
            };
            if !visible.contains(&position) {
                color.fg = color.fg.to_greyscale();
            }
            draw_batch.set(*position, color, to_cp437(c));
        }
    }

    fn render_gui(&mut self, data: &mut RenderSystemData, draw_batch: &mut DrawBatch) {
        // This can go away once the fix for
        // https://github.com/thebracket/bracket-lib/issues/96 released
        let bracket_96_workaround = 1;

        // Draw a box around the main bottom gui panel
        let panel_rect = data.layout.panel();
        draw_batch.draw_box(
            Rect::with_size(
                panel_rect.x1,
                panel_rect.y1,
                panel_rect.width() - bracket_96_workaround,
                panel_rect.height() - bracket_96_workaround,
            ),
            ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
        );

        // Show player health
        let hp_offset: i32 = 12;
        let max_hp_str_len: i32 = 16;
        let hp_bar_offset = hp_offset + max_hp_str_len;

        let stats = (&data.combat_stats, &data.player).join().next().unwrap().0;
        let mut health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        health.truncate(max_hp_str_len.try_into().unwrap());

        draw_batch
            .print_color(
                Point::new(panel_rect.x1 + hp_offset, panel_rect.y1),
                &health,
                ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
            )
            .bar_horizontal(
                Point::new(panel_rect.x1 + hp_bar_offset, panel_rect.y1),
                panel_rect.width() - hp_bar_offset - bracket_96_workaround,
                stats.hp,
                stats.max_hp,
                ColorPair::new(RGB::named(RED), RGB::named(BLACK)),
            );

        // Render game log
        data.gamelog
            .entries
            .iter()
            .rev()
            .take(usize::try_from(panel_rect.height()).unwrap() - 2)
            .enumerate()
            .for_each(|(i, message)| {
                draw_batch.print(
                    Point::new(
                        panel_rect.x1 + 2,
                        panel_rect.y1 + 1 + i32::try_from(i).unwrap(),
                    ),
                    message,
                );
            });

        // Draw mouse cursor
        draw_batch.set_bg(data.mouse_pos.point, RGB::named(MAGENTA));
    }

    fn draw_tooltips(&mut self, data: &mut RenderSystemData, draw_batch: &mut DrawBatch) {
        if !data.map.contains(data.mouse_pos.point.into()) {
            return;
        }

        let player_viewshed = (&data.viewshed, &data.player).join().next().unwrap().0;
        if !player_viewshed
            .visible_tiles
            .contains(&data.mouse_pos.point.into())
        {
            return;
        }

        let tile_contents = data.map.get_tile_contents(data.mouse_pos.point.into());
        if tile_contents.is_none() {
            return;
        }

        let names: Vec<&Name> = tile_contents
            .unwrap()
            .iter()
            .filter_map(|&entity| match data.name.get(entity) {
                Some(name) => Some(name),
                None => None,
            })
            .collect();

        if names.is_empty() {
            return;
        }

        let point_right = data.mouse_pos.point.x > data.layout.width / 2;

        let max_length = names.iter().map(|entry| entry.len()).max().unwrap_or(0);
        let tooltip: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let arrow = if i == 0 {
                    if point_right {
                        " ->"
                    } else {
                        "<- "
                    }
                } else {
                    "   "
                };
                if point_right {
                    format!("{:2$}{}", entry, arrow, max_length)
                } else {
                    format!("{}{:2$}", arrow, entry, max_length)
                }
            })
            .collect();
        let width = tooltip[0].len();

        let x = if point_right {
            (data.mouse_pos.point.x as usize) - width
        } else {
            (data.mouse_pos.point.x as usize) + 1
        };

        for (i, entry) in tooltip.iter().enumerate() {
            draw_batch.print_color(
                Point::new(x, (data.mouse_pos.point.y as usize) + i),
                entry,
                ColorPair::new(RGB::named(WHITE), RGB::named(GREY)),
            );
        }
    }

    fn show_inventory(&mut self, data: &mut RenderSystemData, draw_batch: &mut DrawBatch) {
        if *data.runstate != RunState::ShowInventory {
            return;
        }

        let player_entity = (&data.player, &data.entity).join().next().unwrap().1;
        let inventory: Vec<(&InBackpack, &Name, Entity)> =
            (&data.backpack, &data.name, &data.entity)
                .join()
                .filter(|item| item.0.owner == player_entity)
                .collect();
        let count = inventory.len();

        let inventory_rect = data.layout.inventory(count);
        draw_batch
            .draw_box(
                inventory_rect,
                ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
            )
            .print_color(
                *inventory_rect.position(Vector::new(3, 0)),
                "Inventory",
                ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
            )
            .print_color(
                *inventory_rect.position(Vector::new(3, -1)),
                "ESCAPE to cancel",
                ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
            );

        if count > 0 {
            let mut text_builder = TextBuilder::empty();
            for (j, (_, name, _)) in inventory.iter().enumerate() {
                text_builder
                    .fg(RGB::named(WHITE))
                    .bg(RGB::named(BLACK))
                    .append("(")
                    .fg(RGB::named(YELLOW))
                    .append(&to_char((to_cp437('a') + (j as u16)).try_into().unwrap()).to_string())
                    .fg(RGB::named(WHITE))
                    .append(") ")
                    .append(name)
                    .ln();
            }
            let mut text_block = TextBlock::new(
                inventory_rect.x1 + 2,
                inventory_rect.y1 + 2,
                inventory_rect.width() - 2,
                inventory_rect.height() - 2,
            );
            text_block.print(&text_builder);
            text_block.render_to_draw_batch(draw_batch);
        }

        let shown_entities: Vec<Entity> = inventory.iter().map(|x| x.2).collect();
        *data.shown_inventory = shown_entities.into();
    }
}
