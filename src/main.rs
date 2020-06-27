use bracket_lib::prelude::*;
use core::convert::TryInto;
use specs::prelude::*;

use crate::{
    resources::{gamelog::GameLog, input::Input, layout::Layout, map::Map, runstate::RunState},
    systems::{
        ai::AISystem, damage_system::DamageSystem, death::DeathSystem,
        item_collection::ItemCollectionSystem, item_drop::ItemDropSystem, item_use::ItemUseSystem,
        map_indexing::MapIndexingSystem, mapgen::MapgenSystem, melee_combat::MeleeCombatSystem,
        player_action::PlayerActionSystem, render::RenderSystem, spawner::SpawnerSystem,
        visibility::VisibilitySystem,
    },
};

mod components;
mod lib;
mod resources;
mod systems;

struct Dispatchers {
    main: Dispatcher<'static, 'static>,
    player_action: Dispatcher<'static, 'static>,
    mapgen: Dispatcher<'static, 'static>,
}

struct State {
    world: World,
    dispatchers: Dispatchers,
}

impl GameState for State {
    fn tick(&mut self, mut term: &mut BTerm) {
        self.world.insert(Input::from(&*term));

        let runstate = *self.world.fetch::<RunState>();
        let maybe_newrunstate = match runstate {
            RunState::PreRun => {
                self.dispatchers.main.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
            RunState::AwaitingInput
            | RunState::ShowInventory
            | RunState::ShowDropItem
            | RunState::ShowTargeting { .. } => {
                self.dispatchers.player_action.dispatch(&self.world);
                None
            }
            RunState::PlayerTurn => {
                self.dispatchers.main.dispatch(&self.world);
                Some(RunState::MonsterTurn)
            }
            RunState::MonsterTurn => {
                self.dispatchers.main.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
        };

        if let Some(newrunstate) = maybe_newrunstate {
            *self.world.write_resource::<RunState>() = newrunstate;
        }

        render_draw_buffer(&mut term).unwrap();
        self.world.maintain();
    }
}

fn main() {
    // Initialize bracket-lib
    let term = {
        let mut term = BTermBuilder::simple80x50()
            .with_title("Roguelike Tutorial")
            .build()
            .unwrap();
        term.with_post_scanlines(true);
        term
    };

    // Initialize specs
    let mut gs = State {
        world: World::new(),
        dispatchers: Dispatchers {
            main: DispatcherBuilder::new()
                .with(AISystem, "ai", &[])
                .with(VisibilitySystem, "visibility", &["ai"])
                .with(ItemCollectionSystem, "item_collection", &["ai"])
                .with(ItemDropSystem, "item_drop", &["ai"])
                .with(ItemUseSystem, "item_use", &["ai"])
                .with(MeleeCombatSystem, "melee", &["ai"])
                .with(DamageSystem, "damage", &["melee"])
                .with(DeathSystem, "death", &["damage"])
                .with(
                    MapIndexingSystem,
                    "map_indexing",
                    &["death", "item_collection"],
                )
                .with_barrier()
                .with(RenderSystem, "render", &[])
                .build(),
            player_action: DispatcherBuilder::new()
                .with(PlayerActionSystem, "player_action", &[])
                .with(RenderSystem, "render", &["player_action"])
                .build(),
            mapgen: DispatcherBuilder::new()
                .with(MapgenSystem, "mapgen", &[])
                .with(SpawnerSystem::default(), "spawner", &["mapgen"])
                .build(),
        },
    };

    gs.dispatchers.main.setup(&mut gs.world);
    gs.dispatchers.player_action.setup(&mut gs.world);
    gs.dispatchers.mapgen.setup(&mut gs.world);

    // Create UI layout
    let layout = {
        let (width, height) = term.get_char_size();
        Layout {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            panel_height: 7,
        }
    };
    gs.world.insert(layout);

    // Invoke RNG
    gs.world.insert(RandomNumberGenerator::new());

    // Generate map
    gs.world.insert({
        let map_rect = layout.map();
        Map::new(map_rect.width(), map_rect.height())
    });
    gs.dispatchers.mapgen.dispatch(&gs.world);

    // Welcome!
    gs.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    // And go!
    gs.world.maintain();
    main_loop(term, gs).unwrap();
}
