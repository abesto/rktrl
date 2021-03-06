use core::convert::TryInto;
use std::collections::HashMap;
use std::panic;

use bracket_lib::prelude::*;
use crossbeam_queue::SegQueue;
use legion::{query::component, IntoQuery, Resources, Schedule, World};

use crate::cause_and_effect::{cae_clear_system, cae_debug_system, CauseAndEffect};
use crate::{
    components::{Player, Position, Viewshed},
    resources::{
        FrameData, GameLog, Input, Layout, Map, RexAssets, RunState, RunStateQueue, ShownInventory,
    },
    systems::{
        ai::{ai_system, AiSystemState},
        damage::{damage_system, DamageSystemState},
        death::{death_system, DeathSystemState},
        entity_cleanup::{entity_cleanup_system, EntityCleanupRequest},
        game_log::{game_log_system, GameLogSystemState},
        hunger::{hunger_system, HungerSystemState},
        item_collection::{item_collection_system, ItemCollectionSystemState},
        item_drop::{item_drop_system, ItemDropSystemState},
        item_remove::{item_remove_system, ItemRemoveSystemState},
        item_use::{item_use_system, ItemUseSystemState},
        map_indexing::map_indexing_system,
        mapgen::mapgen_system,
        melee_combat::{melee_combat_system, MeleeCombatSystemState},
        movement::{movement_system, MovementSystemState},
        next_level::{next_level_system, NextLevelSystemState},
        particle::{particle_system, ParticleSystemState},
        player_action::player_action_system,
        render::render_system,
        trigger::{trigger_system, TriggerSystemState},
        turn::turn_system,
        visibility::visibility_system,
    },
    util::saveload,
};

#[macro_use]
extern crate paste;

bracket_terminal::add_wasm_support!();

#[macro_use]
mod cause_and_effect;
mod components;
mod mapgen;
mod resources;
mod systems;
mod util;

#[derive(Eq, PartialEq, Hash, Clone)]
enum ScheduleType {
    Main,
    PlayerAction,
    Mapgen,
    Load,
    RenderOnly,
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
            SegQueue<EntityCleanupRequest>,
            RexAssets,
            RunStateQueue
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

enum NewRunState {
    PushFront(RunState),
    PushBack(RunState),
    None,
}

impl GameState for State {
    fn tick(&mut self, mut term: &mut BTerm) {
        self.resources.insert(Input::from(&*term));
        self.resources.insert(FrameData::from(&*term));

        {
            let maybe_new_runstate = self
                .resources
                .get_mut_or_default::<RunStateQueue>()
                .pop_front();
            if let Some(new_runstate) = maybe_new_runstate {
                self.resources.insert(new_runstate);
            }
        }

        let runstate = self.resources.get_or_default::<RunState>().clone();
        let maybe_new_runstate = match runstate {
            RunState::PreRun => {
                self.reset();
                self.resources.insert(GameLog {
                    entries: vec!["Welcome to Rusty Roguelike".to_string()],
                });
                self.execute(ScheduleType::Mapgen);
                NewRunState::PushBack(RunState::AwaitingInput)
            }
            RunState::NextLevel => {
                self.resources.get_mut_or_default::<RunStateQueue>().clear();
                self.execute(ScheduleType::Mapgen);
                NewRunState::PushBack(RunState::AwaitingInput)
            }
            RunState::AwaitingInput
            | RunState::ShowInventory
            | RunState::ShowDropItem
            | RunState::ShowRemoveItem
            | RunState::MainMenu { .. }
            | RunState::ShowTargeting { .. } => {
                self.execute(ScheduleType::PlayerAction);
                NewRunState::None
            }
            RunState::PlayerTurn => {
                self.execute(ScheduleType::Main);
                NewRunState::PushBack(RunState::MonsterTurn)
            }
            RunState::MonsterTurn => {
                self.execute(ScheduleType::Main);
                NewRunState::PushBack(RunState::AwaitingInput)
            }
            RunState::SaveGame => {
                saveload::save(&self.world, &self.resources);
                self.reset();
                NewRunState::PushBack(RunState::default())
            }
            RunState::LoadGame => {
                self.reset();
                saveload::load(&mut self.world, &mut self.resources);
                self.execute(ScheduleType::Load);
                NewRunState::PushBack(RunState::AwaitingInput)
            }
            RunState::GameOver => {
                self.reset();
                self.execute(ScheduleType::PlayerAction);
                NewRunState::None
            }
            RunState::MagicMapReveal { row } => {
                let done = {
                    let map = self.resources.get_mut::<Map>().unwrap();
                    let (viewshed,) = <(&mut Viewshed,)>::query()
                        .filter(component::<Player>())
                        .iter_mut(&mut self.world)
                        .next()
                        .unwrap();
                    for x in 0..map.width {
                        viewshed.revealed_tiles.insert(Position::new(x, row));
                    }
                    row == map.height - 1
                };
                self.execute(ScheduleType::RenderOnly);
                if done {
                    NewRunState::PushBack(RunState::MonsterTurn)
                } else {
                    NewRunState::PushFront(RunState::MagicMapReveal { row: row + 1 })
                }
            }
            RunState::MapGeneration {
                mut snapshots,
                final_map,
                timer,
            } => {
                let retval = if timer > 500.0 {
                    if let Some(snapshot) = snapshots.pop_front() {
                        println!("Displaying snapshot, {} more left", snapshots.len());
                        self.resources.insert(snapshot);
                        NewRunState::PushFront(RunState::MapGeneration {
                            snapshots,
                            final_map,
                            timer: 0.0,
                        })
                    } else {
                        self.resources.insert(final_map);
                        NewRunState::None
                    }
                } else {
                    NewRunState::PushFront(RunState::MapGeneration {
                        snapshots,
                        final_map,
                        timer: timer + term.frame_time_ms,
                    })
                };
                self.execute(ScheduleType::RenderOnly);
                retval
            }
        };

        {
            let mut queue = self.resources.get_mut_or_default::<RunStateQueue>();
            match maybe_new_runstate {
                NewRunState::PushBack(state) => queue.push_back(state),
                NewRunState::PushFront(state) => queue.push_front(state),
                _ => {}
            }
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
            .add_system(ai_system(AiSystemState::new(&resources)))
            .flush()
            .add_system(movement_system(MovementSystemState::new(&resources)))
            .add_system(trigger_system(TriggerSystemState::new(&resources)))
            .flush()
            .add_system(visibility_system())
            .add_system(item_collection_system(ItemCollectionSystemState::new(
                &resources,
            )))
            .add_system(item_drop_system(ItemDropSystemState::new(&resources)))
            .add_system(item_use_system(ItemUseSystemState::new(&resources)))
            .add_system(item_remove_system(ItemRemoveSystemState::new(&resources)))
            .add_system(melee_combat_system(MeleeCombatSystemState::new(&resources)))
            .flush()
            .add_system(hunger_system(HungerSystemState::new(&resources)))
            .flush()
            .add_system(damage_system(DamageSystemState::new(&resources)))
            .flush()
            .add_system(death_system(DeathSystemState::new(&resources)))
            .flush()
            .add_system(map_indexing_system())
            .add_system(particle_system(ParticleSystemState::new(&resources)))
            .flush()
            .add_system(render_system())
            .add_system(game_log_system(GameLogSystemState::new(&resources)))
            .add_system(cae_debug_system())
            .add_system(cae_clear_system())
            .add_system(entity_cleanup_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::PlayerAction,
        Schedule::builder()
            .add_system(cae_clear_system())
            .add_system(turn_system())
            .add_system(player_action_system())
            .flush()
            .add_system(next_level_system(NextLevelSystemState::new(&resources)))
            .add_system(particle_system(ParticleSystemState::new(&resources)))
            .flush()
            .add_system(render_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::Mapgen,
        Schedule::builder()
            .add_system(mapgen_system())
            .flush()
            .add_system(map_indexing_system())
            .add_system(visibility_system())
            .add_system(game_log_system(GameLogSystemState::new(&resources)))
            .add_system(cae_debug_system())
            .add_system(cae_clear_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::Load,
        Schedule::builder()
            .add_system(map_indexing_system())
            .build(),
    );
    schedules.insert(
        ScheduleType::RenderOnly,
        Schedule::builder()
            .add_system(visibility_system())
            .add_system(render_system())
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
