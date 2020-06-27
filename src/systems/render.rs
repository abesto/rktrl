use std::convert::{TryFrom, TryInto};

use bracket_lib::prelude::*;
use shipyard::*;

use crate::{
    components::{
        combat_stats::CombatStats, in_backpack::InBackpack, name::Name, player::Player,
        position::Position, renderable::Renderable, viewshed::Viewshed,
    },
    resources::{
        gamelog::GameLog,
        layout::Layout,
        map::{Map, TileType},
        runstate::RunState,
        shown_inventory::ShownInventory,
    },
};

pub fn render(
    ref entities: EntitiesView,

    ref players: View<Player>,
    ref viewsheds: View<Viewshed>,
    ref positions: View<Position>,
    ref renderables: View<Renderable>,
    ref statses: View<CombatStats>,
    ref names: View<Name>,
    ref backpacks: View<InBackpack>,

    map: UniqueView<Map>,
    layout: UniqueView<Layout>,
    runstate: UniqueView<RunState>,

    mut shown_inventory: UniqueView<ShownInventory>,
    mut gamelog: UniqueViewMut<GameLog>,
    mut term: UniqueViewMut<BTerm>,
) {
    let player_viewshed = (players, viewsheds).iter().next().unwrap().1;
    let visible = player_viewshed.visible_tiles;
    let revealed = player_viewshed.revealed_tiles;

    // Clear the screen first
    term.cls();

    // Render entities
    {
        for (position, renderable) in (positions, renderables).iter() {
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

    // Render map
    {
        for position in revealed {
            let tile = map[&position];
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

    // Render GUI
    {
        // This can go away once the fix for
        // https://github.com/thebracket/bracket-lib/issues/96 released
        let bracket_96_workaround = 1;

        // Draw a box around the main bottom gui panel
        let panel_rect = layout.panel();
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

        let stats = (statses, players).iter().next().unwrap().0;
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
        gamelog
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

        // Draw mouse cursor
        let mouse_pos = term.mouse_pos();
        term.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    };

    // Draw tooltips
    {
        let mouse_pos = Position::from(term.mouse_pos());
        if !map.contains(mouse_pos) {
            return;
        }

        if !visible.contains(&mouse_pos) {
            return;
        }

        let tile_contents = map.get_tile_contents(mouse_pos);
        if tile_contents.is_none() {
            return;
        }

        let names: Vec<String> = tile_contents
            .unwrap()
            .iter()
            .filter_map(|&entity| names.get(entity).map(ToString::to_string).ok())
            .collect();

        if names.is_empty() {
            return;
        }

        let point_right = mouse_pos.x > layout.width / 2;

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
            (mouse_pos.x as usize) - width
        } else {
            (mouse_pos.x as usize) + 1
        };

        for (i, entry) in tooltip.iter().enumerate() {
            term.print_color(
                x,
                (mouse_pos.y as usize) + i,
                RGB::named(WHITE),
                RGB::named(GREY),
                entry,
            );
        }
    }

    // Show inventory
    {
        if *runstate != RunState::ShowInventory {
            return;
        }

        let player_entity = players.iter().with_id().next().unwrap().0;
        let inventory: Vec<(&InBackpack, &Name, EntityId)> = (backpacks, names)
            .iter()
            .filter(|item| item.0.owner == player_entity)
            .with_id()
            .map(|(id, (backpack, name))| (backpack, name, id))
            .collect();
        let count = inventory.len();

        let mut y = (25 - (count / 2)) as i32;
        term.draw_box(
            15,
            y - 2,
            31,
            (count + 3) as i32,
            RGB::named(WHITE),
            RGB::named(BLACK),
        );
        term.print_color(
            18,
            y - 2,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "Inventory",
        );
        term.print_color(
            18,
            y + count as i32 + 1,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "ESCAPE to cancel",
        );

        for (j, (_, name, _)) in inventory.iter().enumerate() {
            term.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
            term.set(
                18,
                y,
                RGB::named(YELLOW),
                RGB::named(BLACK),
                97 + j as FontCharType,
            );
            term.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

            term.print(21, y, name.to_string());
            y += 1;
        }

        let shown_entities: Vec<EntityId> = inventory.iter().map(|x| x.2).collect();
        *shown_inventory = shown_entities.into();
    }
}
