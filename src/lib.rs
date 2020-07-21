use core::convert::TryInto;
use std::panic;

use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::{
    resources::{FrameData, GameLog, Input, Layout, Map, RunState, RunStateQueue},
    systems::{
        ai::AISystem,
        damage_system::DamageSystem,
        death::DeathSystem,
        item_collection::ItemCollectionSystem,
        item_drop::ItemDropSystem,
        item_remove::ItemRemoveSystem,
        item_use::ItemUseSystem,
        map_indexing::MapIndexingSystem,
        mapgen::MapgenSystem,
        melee_combat::MeleeCombatSystem,
        next_level::NextLevelSystem,
        particle::ParticleSystem,
        player_action::PlayerActionSystem,
        render::RenderSystem,
        saveload::{LoadSystem, SaveSystem},
        spawner::SpawnerSystem,
        visibility::VisibilitySystem,
    },
};

bracket_terminal::add_wasm_support!();

mod components;
mod resources;
mod systems;
mod util;

struct Dispatchers {
    main: Dispatcher<'static, 'static>,
    player_action: Dispatcher<'static, 'static>,
    mapgen: Dispatcher<'static, 'static>,
    save: Dispatcher<'static, 'static>,
    load: Dispatcher<'static, 'static>,
}

struct State {
    world: World,
    dispatchers: Dispatchers,
}

impl State {
    fn reset(&mut self) {
        // Probably this would be cleaner as a system, but whatevs
        self.world.delete_all();
        self.world.fetch_mut::<GameLog>().entries.clear();
        self.world.insert({
            let map_rect = self.world.fetch::<Layout>().map();
            Map::new(map_rect.width(), map_rect.height(), 1)
        });
        self.world.insert(GameLog {
            entries: vec!["Welcome to Rusty Roguelike".to_string()],
        });
    }
}

impl GameState for State {
    fn tick(&mut self, mut term: &mut BTerm) {
        self.world.insert(Input::from(&*term));
        self.world.insert(FrameData::from(&*term));

        let maybe_new_runstate = self.world.fetch_mut::<RunStateQueue>().pop_front();
        if let Some(new_runstate) = maybe_new_runstate {
            self.world.insert(new_runstate);
        }

        let runstate = *self.world.fetch::<RunState>();
        let maybe_newrunstate = match runstate {
            RunState::PreRun => {
                self.reset();
                self.dispatchers.mapgen.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
            RunState::NextLevel => {
                self.dispatchers.mapgen.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
            RunState::AwaitingInput
            | RunState::ShowInventory
            | RunState::ShowDropItem
            | RunState::ShowRemoveItem
            | RunState::MainMenu { .. }
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
            RunState::SaveGame => {
                SaveSystem::prepare(&mut self.world);
                self.dispatchers.save.dispatch(&self.world);
                Some(RunState::default())
            }
            RunState::LoadGame => {
                self.reset();
                self.dispatchers.load.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
            RunState::GameOver => {
                self.dispatchers.player_action.dispatch(&self.world);
                None
            }
        };

        if let Some(newrunstate) = maybe_newrunstate {
            self.world.insert(newrunstate);
        }

        render_draw_buffer(&mut term).unwrap();
        self.world.maintain();
    }
}

pub fn main() -> BError {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Initialize bracket-util
    let term = {
        let mut term = BTermBuilder::simple80x50()
            .with_title("Roguelike Tutorial")
            .build()?;
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
                .with(ItemRemoveSystem, "item_remove", &["ai"])
                .with(MeleeCombatSystem, "melee", &["ai"])
                .with(DamageSystem, "damage", &["melee"])
                .with(DeathSystem, "death", &["damage"])
                .with(
                    MapIndexingSystem,
                    "map_indexing",
                    &["death", "item_collection"],
                )
                .with_barrier()
                .with(ParticleSystem, "particles", &[])
                .with(RenderSystem, "render", &["particles"])
                .build(),
            player_action: DispatcherBuilder::new()
                .with(PlayerActionSystem, "player_action", &[])
                .with(ParticleSystem, "particles", &["player_action"])
                .with(RenderSystem, "render", &["player_action", "particles"])
                .build(),
            mapgen: DispatcherBuilder::new()
                .with(NextLevelSystem, "cleanup", &[])
                .with(MapgenSystem, "mapgen", &[])
                .with(SpawnerSystem::default(), "spawner", &["mapgen", "cleanup"])
                .with(VisibilitySystem, "visibility", &["spawner"])
                .build(),
            save: DispatcherBuilder::new()
                .with(SaveSystem, "save", &[])
                .build(),
            load: DispatcherBuilder::new()
                .with(LoadSystem, "load", &[])
                .with(MapIndexingSystem, "map_indexing", &["load"])
                .build(),
        },
    };

    gs.dispatchers.main.setup(&mut gs.world);
    gs.dispatchers.player_action.setup(&mut gs.world);
    gs.dispatchers.mapgen.setup(&mut gs.world);
    gs.dispatchers.save.setup(&mut gs.world);
    gs.dispatchers.load.setup(&mut gs.world);

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

    // And go!
    gs.reset();
    gs.world.maintain();
    main_loop(term, gs)
}
