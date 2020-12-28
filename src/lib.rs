use core::convert::TryInto;
use std::collections::HashMap;
use std::panic;

use bracket_lib::prelude::*;
use crossbeam_queue::SegQueue;
use legion::{Resources, Schedule, World};

use crate::cause_and_effect::{cae_clear_system, cae_debug_system, CauseAndEffect};
use crate::systems::damage::DamageSystemState;
use crate::{
    resources::{FrameData, GameLog, Input, Layout, Map, RunState, RunStateQueue, ShownInventory},
    systems::{
        ai::ai_system,
        damage::damage_system,
        death::death_system,
        hunger::{hunger_system, HungerSystemState},
        item_collection::item_collection_system,
        item_drop::item_drop_system,
        item_remove::item_remove_system,
        item_use::item_use_system,
        map_indexing::map_indexing_system,
        mapgen::mapgen_system,
        melee_combat::{melee_combat_system, MeleeCombatSystemState},
        movement::{movement_system, MovementSystemState},
        next_level::next_level_system,
        particle::{particle_system, ParticleSystemState},
        player_action::player_action_system,
        render::render_system,
        spawner::{spawner_system, SpawnRequest},
        turn::turn_system,
        visibility::visibility_system,
    },
    util::saveload,
};

bracket_terminal::add_wasm_support!();

mod cause_and_effect;
mod components;
mod resources;
mod systems;
mod util;

#[derive(Eq, PartialEq, Hash, Clone)]
enum ScheduleType {
    Main,
    PlayerAction,
    Mapgen,
    Load,
}

type Schedules = HashMap<ScheduleType, Schedule>;

struct State {
    world: World,
    resources: Resources,
    schedules: Schedules,
}

macro_rules! insert_default_resources {
    ($resources:expr, [$($types:ty),* $(,)?]) => {
        $(
            $resources.insert(<$types>::default());
        )*
    }
}

impl State {
    fn reset(&mut self) {
        // Probably this would be cleaner as a system, but whatever
        self.world.clear();
        // TODO check if inserting the same type of resource over and over again leads to memleak
        self.resources.insert({
            let map_rect = self.resources.get::<Layout>().unwrap().map();
            Map::new(map_rect.width(), map_rect.height(), 1)
        });
        insert_default_resources!(self.resources, [
            GameLog,
            ShownInventory,
            SegQueue<SpawnRequest>,
        ]);
        self.resources
            .get_mut::<CauseAndEffect>()
            .unwrap()
            .new_turn();
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
                self.resources.insert(GameLog {
                    entries: vec!["Welcome to Rusty Roguelike".to_string()],
                });
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
                saveload::save(&mut self.world, &self.resources);
                self.reset();
                Some(RunState::default())
            }
            RunState::LoadGame => {
                self.reset();
                saveload::load(&mut self.world, &mut self.resources);
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
    resources.insert(CauseAndEffect::default());
    let mut schedules = HashMap::new();
    schedules.insert(
        ScheduleType::Main,
        Schedule::builder()
            .add_system(turn_system())
            .add_system(ai_system())
            .flush()
            .add_system(movement_system(MovementSystemState::new(&resources)))
            .add_system(visibility_system())
            .add_system(item_collection_system())
            .add_system(item_drop_system())
            .add_system(item_use_system())
            .add_system(item_remove_system())
            .add_system(melee_combat_system(MeleeCombatSystemState::new(&resources)))
            .flush()
            .add_system(hunger_system(HungerSystemState::new(&resources)))
            .flush()
            .add_system(damage_system(DamageSystemState::new(&resources)))
            .flush()
            .add_system(death_system())
            .flush()
            .add_system(map_indexing_system())
            .add_system(particle_system(ParticleSystemState::new(&resources)))
            .flush()
            .add_system(render_system())
            .add_system(cae_debug_system())
            .add_system(cae_clear_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::PlayerAction,
        Schedule::builder()
            .add_system(cae_clear_system())
            .add_system(turn_system())
            .add_system(player_action_system())
            .flush()
            .add_system(particle_system(ParticleSystemState::new(&resources)))
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
            .add_system(spawner_system())
            .flush()
            .add_system(visibility_system())
            .add_system(map_indexing_system())
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
