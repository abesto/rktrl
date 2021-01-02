use std::collections::HashSet;
use std::convert::TryFrom;
use std::convert::TryInto;

use bracket_lib::prelude::*;
use legion::{component, system, world::SubWorld, Entity, EntityStore, IntoQuery};
use strum::IntoEnumIterator;

use crate::util::world_ext::WorldExt;
use crate::{
    components::*,
    resources::{Input, *},
    util::{rect_ext::RectExt, vector::Vector},
};

#[system]
#[read_component(Entity)]
#[read_component(AreaOfEffect)]
#[read_component(CombatStats)]
#[read_component(Equipped)]
#[read_component(InBackpack)]
#[read_component(Name)]
#[read_component(Player)]
#[read_component(Position)]
#[read_component(Renderable)]
#[read_component(Viewshed)]
#[read_component(HungerClock)]
#[allow(clippy::too_many_arguments)]
pub fn render(
    world: &SubWorld,
    #[resource] game_log: &GameLog,
    #[resource] run_state: &RunState,
    #[resource] layout: &Layout,
    #[resource] map: &Map,
    #[resource] input: &Input,
    #[resource] shown_inventory: &mut ShownInventory,
    #[resource] rex_assets: &RexAssets,
) {
    let draw_batch = &mut DrawBatch::new();
    draw_batch.cls();
    match *run_state {
        RunState::MainMenu { .. } => render_main_menu(run_state, draw_batch, rex_assets),
        RunState::GameOver => render_game_over(draw_batch),
        _ => {
            render_map(world, map, draw_batch);
            render_entities(world, draw_batch);
            render_gui(world, map, layout, game_log, input, draw_batch);
            targeting_overlay(world, run_state, map, input, draw_batch);
            draw_tooltips(world, map, layout, input, draw_batch);
            show_inventory(world, run_state, layout, shown_inventory, draw_batch);
        }
    };
    draw_batch.submit(0).unwrap();
}

fn player_visible_tiles(world: &SubWorld) -> HashSet<Position> {
    <(&Viewshed,)>::query()
        .filter(component::<Player>())
        .iter(world)
        .flat_map(|(viewshed,)| viewshed.visible_tiles.clone())
        .collect()
}

fn player_revealed_tiles(world: &SubWorld) -> HashSet<Position> {
    <(&Viewshed,)>::query()
        .filter(component::<Player>())
        .iter(world)
        .flat_map(|(viewshed,)| viewshed.revealed_tiles.clone())
        .collect()
}

fn render_entities(world: &SubWorld, draw_batch: &mut DrawBatch) {
    let visible = player_visible_tiles(world);
    let mut data = <(&Position, &Renderable)>::query()
        .iter(world)
        .collect::<Vec<_>>();
    data.sort_unstable_by_key(|r| &r.1.render_order);
    for (position, renderable) in data {
        if !visible.contains(position) {
            continue;
        }
        draw_batch.set(**position, renderable.color, renderable.glyph);
    }
}

fn render_map(world: &SubWorld, map: &Map, draw_batch: &mut DrawBatch) {
    let visible = player_visible_tiles(world);
    let revealed = player_revealed_tiles(world);

    for position in &revealed {
        let tile = map[&position];
        let (fg_candidate, glyph) = match tile {
            TileType::Floor => (RGB::named(GRAY50), to_cp437('.')),
            TileType::Wall => (RGB::named(GREEN), map.wall_glyph(*position, &revealed)),
            TileType::DownStairs => (RGB::named(CYAN), to_cp437('>')),
        };
        let (fg, bg) = if visible.contains(&position) {
            (
                fg_candidate,
                if map.has_bloodstain(*position) {
                    RGB::from_f32(0.75, 0., 0.)
                } else {
                    RGB::named(BLACK)
                },
            )
        } else {
            (fg_candidate.to_greyscale(), RGB::named(BLACK))
        };
        draw_batch.set(**position, ColorPair::new(fg, bg), glyph);
    }
}

fn render_gui(
    world: &SubWorld,
    map: &Map,
    layout: &Layout,
    game_log: &GameLog,
    input: &Input,
    draw_batch: &mut DrawBatch,
) {
    // This can go away once the fix for
    // https://github.com/thebracket/bracket-lib/issues/96 released
    let bracket_96_workaround = 1;

    // Draw a box around the main bottom gui panel
    let panel_rect = layout.panel();
    draw_batch.draw_box(
        Rect::with_size(
            panel_rect.x1,
            panel_rect.y1,
            panel_rect.width() - bracket_96_workaround,
            panel_rect.height() - bracket_96_workaround,
        ),
        ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
    );

    // Show depth
    draw_batch.print_color(
        Point::new(panel_rect.x1 + 2, panel_rect.y1),
        format!("Depth: {}", map.depth),
        ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
    );

    // Show player health
    let hp_offset: i32 = 12;
    let max_hp_str_len: i32 = 16;
    let hp_bar_offset = hp_offset + max_hp_str_len;

    let (stats,) = <(&CombatStats,)>::query()
        .filter(component::<Player>())
        .iter(world)
        .next()
        .unwrap();
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
    game_log
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

    // Hunger status
    if let Some((clock,)) = <(&HungerClock,)>::query()
        .filter(component::<Player>())
        .iter(world)
        .next()
    {
        if let Some((fg, text)) = match clock.state {
            HungerState::WellFed => Some((GREEN, "Well Fed")),
            HungerState::Normal => None,
            HungerState::Hungry => Some((ORANGE, "Hungry")),
            HungerState::Starving => Some((RED, "Starving")),
        } {
            draw_batch.print_color(
                layout.hunger_status(text.len() as i32),
                text,
                ColorPair::new(RGB::named(fg), RGB::named(BLACK)),
            );
        }
    }

    // Draw mouse cursor
    draw_batch.set_bg(input.mouse_pos, RGB::named(MAGENTA));
}

fn draw_tooltips(
    world: &SubWorld,
    map: &Map,
    layout: &Layout,
    input: &Input,
    draw_batch: &mut DrawBatch,
) {
    if !map.contains(input.mouse_pos.into()) {
        return;
    }

    if !player_visible_tiles(&world).contains(&input.mouse_pos.into()) {
        return;
    }

    let tile_contents = map.get_tile_contents(input.mouse_pos.into());
    if tile_contents.is_none() {
        return;
    }

    let names: Vec<String> = tile_contents
        .unwrap()
        .iter()
        .filter_map(|&entity| {
            world
                .entry_ref(entity)
                .ok()
                .and_then(|entry| entry.get_component().ok().map(Name::to_string))
        })
        .collect();

    if names.is_empty() {
        return;
    }

    let point_right = input.mouse_pos.x > layout.width / 2;

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
        (input.mouse_pos.x as usize) - width
    } else {
        (input.mouse_pos.x as usize) + 1
    };

    for (i, entry) in tooltip.iter().enumerate() {
        draw_batch.print_color(
            Point::new(x, (input.mouse_pos.y as usize) + i),
            entry,
            ColorPair::new(RGB::named(WHITE), RGB::named(GREY)),
        );
    }
}

fn show_inventory(
    world: &SubWorld,
    run_state: &RunState,
    layout: &Layout,
    shown_inventory: &mut ShownInventory,
    draw_batch: &mut DrawBatch,
) {
    if !run_state.show_inventory() {
        return;
    }

    let title = match *run_state {
        RunState::ShowDropItem => "Drop Which Item?",
        RunState::ShowInventory => "Inventory",
        RunState::ShowRemoveItem => "Remove Which Item?",
        _ => panic!(),
    };

    let player_entity = world.player_entity();
    let inventory: Vec<(&Name, &Entity)> = if *run_state == RunState::ShowRemoveItem {
        <(&Equipped, &Name, Entity)>::query()
            .iter(world)
            .filter(|(equipped, _, _)| equipped.owner == *player_entity)
            .map(|(_, name, entity)| (name, entity))
            .collect()
    } else {
        <(&InBackpack, &Name, Entity)>::query()
            .iter(world)
            .filter(|(in_backpack, _, _)| in_backpack.owner == *player_entity)
            .map(|(_, name, entity)| (name, entity))
            .collect()
    };
    let count = inventory.len();
    let max_len = inventory.iter().map(|x| x.0.len()).max().unwrap_or(0);

    let inventory_rect = layout.inventory(count, max_len);
    draw_batch
        .draw_box(
            inventory_rect,
            ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
        )
        .print_color(
            *inventory_rect.position(Vector::new(3, 0)),
            title,
            ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
        )
        .print_color(
            *inventory_rect.position(Vector::new(3, -1)),
            "ESCAPE to cancel",
            ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
        );

    if count > 0 {
        let mut text_builder = TextBuilder::empty();
        for (j, (name, _)) in inventory.iter().enumerate() {
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

    let shown_entities: Vec<Entity> = inventory.iter().map(|x| *x.1).collect();
    *shown_inventory = shown_entities.into();
}

fn targeting_overlay(
    world: &SubWorld,
    run_state: &RunState,
    map: &Map,
    input: &Input,
    draw_batch: &mut DrawBatch,
) {
    if let RunState::ShowTargeting { range, item } = *run_state {
        draw_batch.print_color(
            Point::new(5, 0),
            "Select Target:",
            ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
        );

        // Highlight available target cells
        let (viewshed, &player_pos) = <(&Viewshed, &Position)>::query()
            .filter(component::<Player>())
            .iter(world)
            .next()
            .unwrap();

        let range_f32 = range as f32;
        let available_cells = &viewshed
            .visible_tiles
            .iter()
            .map(|&position| *position)
            .filter(|point| DistanceAlg::Pythagoras.distance2d(*player_pos, *point) <= range_f32)
            .collect::<Vec<Point>>();

        for tile in available_cells {
            draw_batch.set_bg(*tile, RGB::named(BLUE));
        }

        let valid_aim = available_cells.contains(&input.mouse_pos);

        // Highlight AoE, if applicable
        if valid_aim {
            if let Ok(aoe) = world
                .entry_ref(item)
                .unwrap()
                .get_component::<AreaOfEffect>()
            {
                let affected_cells = field_of_view(input.mouse_pos, aoe.radius, map)
                    .iter()
                    .filter(|&p| viewshed.revealed_tiles.contains(&Position::from(*p)))
                    .cloned()
                    .collect::<Vec<_>>();
                for cell in affected_cells {
                    draw_batch.set_bg(cell, RGB::named(DARK_GOLDENROD));
                }
            }
        }

        // Draw mouse cursor
        draw_batch.set_bg(
            input.mouse_pos,
            if valid_aim {
                RGB::named(CYAN)
            } else {
                RGB::named(RED)
            },
        );
    };
}

fn render_main_menu(run_state: &RunState, draw_batch: &mut DrawBatch, rex_assets: &RexAssets) {
    if let RunState::MainMenu {
        selection,
        load_enabled,
    } = *run_state
    {
        crate::util::bracket_lib_ext::xp_to_draw_batch(&rex_assets.menu, draw_batch, 0, 0);
        draw_batch.print_color_centered(
            15,
            "Rust Roguelike Tutorial",
            ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
        );

        for (i, item) in MainMenuSelection::iter().enumerate() {
            draw_batch.print_color_centered(
                24 + i,
                item,
                ColorPair::new(
                    if selection == item {
                        RGB::named(MAGENTA)
                    } else if !load_enabled && item == MainMenuSelection::LoadGame {
                        RGB::named(GRAY)
                    } else {
                        RGB::named(WHITE)
                    },
                    RGB::named(BLACK),
                ),
            );
        }
    }
}

fn render_game_over(draw_batch: &mut DrawBatch) {
    draw_batch.print_color_centered(
        15,
        "Your journey has ended!",
        ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
    );
    draw_batch.print_color_centered(
        17,
        "One day, we'll tell you all about how you did.",
        ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
    );
    draw_batch.print_color_centered(
        18,
        "That day, sadly, is not in this chapter...",
        ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)),
    );

    draw_batch.print_color_centered(
        20,
        "Press any key to return to the menu.",
        ColorPair::new(RGB::named(MAGENTA), RGB::named(BLACK)),
    );
}
