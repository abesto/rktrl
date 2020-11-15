use core::convert::TryInto;
use std::panic;

use bracket_lib::prelude::*;
use legion::{Resources, Schedule, World};

use crate::systems::spawner::SpawnerSystemState;
use crate::{
    resources::{FrameData, GameLog, Input, Layout, Map, RunState, RunStateQueue, ShownInventory},
    systems::{
        ai::ai_system,
        damage_system::damage_system,
        death::death_system,
        hunger::hunger_system,
        item_collection::item_collection_system,
        item_drop::item_drop_system,
        item_remove::item_remove_system,
        item_use::item_use_system,
        map_indexing::map_indexing_system,
        mapgen::mapgen_system,
        melee_combat::melee_combat_system,
        next_level::next_level_system,
        particle::{particle_system, ParticleRequests},
        player_action::player_action_system,
        render::render_system,
        //saveload::{LoadSystem, SaveSystem},
        spawner::spawner_system,
        visibility::visibility_system,
    },
};
use wasm_bindgen::__rt::std::collections::HashMap;

bracket_terminal::add_wasm_support!();

mod components;
mod resources;
mod systems;
mod util;

#[derive(Eq, PartialEq, Hash, Clone)]
enum ScheduleType {
    Main,
    PlayerAction,
    Mapgen,
    Save,
    Load,
}

type Schedules = HashMap<ScheduleType, Schedule>;

struct State {
    world: World,
    resources: Resources,
    schedules: Schedules,
}

impl State {
    fn reset(&mut self) {
        // Probably this would be cleaner as a system, but whatever
        self.world.clear();
        // TODO verify: probably don't need this as we create a new GameLog below
        self.resources
            .get_mut_or_default::<GameLog>()
            .entries
            .clear();
        self.resources.insert({
            let map_rect = self.resources.get::<Layout>().unwrap().map();
            Map::new(map_rect.width(), map_rect.height(), 1)
        });
        self.resources.insert(GameLog {
            entries: vec!["Welcome to Rusty Roguelike".to_string()],
        });
        self.resources.insert(ShownInventory::default());
        self.resources.insert(ParticleRequests::default());
    }

    fn execute(&mut self, schedule_type: ScheduleType) {
        self.schedules
            .get_mut(&schedule_type)
            .unwrap()
            .execute(&mut self.world, &mut self.resources);
    }
}

impl GameState for State {
    fn tick(&mut self, mut term: &mut BTerm) {
        self.resources.insert(Input::from(&*term));
        self.resources.insert(FrameData::from(&*term));

        let maybe_new_runstate = self
            .resources
            .get_mut_or_default::<RunStateQueue>()
            .pop_front();
        if let Some(new_runstate) = maybe_new_runstate {
            self.resources.insert(new_runstate);
        }

        let runstate = *self.resources.get_or_default::<RunState>();
        let maybe_newrunstate = match runstate {
            RunState::PreRun => {
                self.reset();
                self.execute(ScheduleType::Mapgen);
                Some(RunState::AwaitingInput)
            }
            RunState::NextLevel => {
                self.execute(ScheduleType::Mapgen);
                Some(RunState::AwaitingInput)
            }
            RunState::AwaitingInput
            | RunState::ShowInventory
            | RunState::ShowDropItem
            | RunState::ShowRemoveItem
            | RunState::MainMenu { .. }
            | RunState::ShowTargeting { .. } => {
                self.execute(ScheduleType::PlayerAction);
                None
            }
            RunState::PlayerTurn => {
                self.execute(ScheduleType::Main);
                Some(RunState::MonsterTurn)
            }
            RunState::MonsterTurn => {
                self.execute(ScheduleType::Main);
                Some(RunState::AwaitingInput)
            }
            RunState::SaveGame => {
                //SaveSystem::prepare(&mut self.world);
                self.execute(ScheduleType::Save);
                Some(RunState::default())
            }
            RunState::LoadGame => {
                self.reset();
                self.execute(ScheduleType::Load);
                Some(RunState::AwaitingInput)
            }
            RunState::GameOver => {
                self.execute(ScheduleType::PlayerAction);
                None
            }
        };

        if let Some(newrunstate) = maybe_newrunstate {
            self.resources.insert(newrunstate);
        }

        render_draw_buffer(&mut term).unwrap();
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

    // Initialize Legion ECS

    let mut resources = Resources::default();
    let mut schedules = HashMap::new();
    schedules.insert(
        ScheduleType::Main,
        Schedule::builder()
            .add_system(ai_system())
            .flush()
            .add_system(visibility_system())
            .add_system(item_collection_system())
            .add_system(item_drop_system())
            .add_system(item_use_system())
            .add_system(item_remove_system())
            .add_system(melee_combat_system())
            .flush()
            .add_system(hunger_system())
            .flush()
            .add_system(damage_system())
            .flush()
            .add_system(death_system())
            .flush()
            .add_system(map_indexing_system())
            .add_system(particle_system())
            .flush()
            .add_system(render_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::PlayerAction,
        Schedule::builder()
            .add_system(player_action_system())
            .flush()
            .add_system(particle_system())
            .flush()
            .add_system(render_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::Mapgen,
        Schedule::builder()
            .add_system(next_level_system())
            .add_system(mapgen_system())
            .flush()
            .add_system(spawner_system(SpawnerSystemState::new(&mut resources)))
            .flush()
            .add_system(visibility_system())
            .add_system(map_indexing_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::Save,
        Schedule::builder()
            //.add_system(save_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::Load,
        Schedule::builder()
            //.add_system(load_system())
            .flush()
            .add_system(map_indexing_system())
            .build(),
    );
    let mut gs = State {
        world: World::default(),
        resources,
        schedules,
    };

    // Create UI layout
    let layout = {
        let (width, height) = term.get_char_size();
        Layout {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            panel_height: 7,
        }
    };
    gs.resources.insert(layout);

    // Invoke RNG
    gs.resources.insert(RandomNumberGenerator::new());

    // And go!
    gs.reset();
    main_loop(term, gs)
}
