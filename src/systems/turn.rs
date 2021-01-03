use legion::{query::component, system, world::SubWorld, Entity, IntoQuery};

use crate::cause_and_effect::{CauseAndEffect, Label};
use crate::util::world_ext::WorldExt;
use crate::{components::*, resources::*};

#[system]
#[read_component(Monster)]
#[read_component(Player)]
#[read_component(CombatStats)]
pub fn turn(
    #[resource] run_state: &RunState,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
) {
    // Decide what actors take action this turn
    // This is where the core of an energy system would live
    match *run_state {
        RunState::MonsterTurn => {
            <(Entity,)>::query()
                .filter(component::<Monster>())
                .for_each(world, |(&actor,)| {
                    cae.add_effect(&cae.get_root(), Label::Turn { actor });
                });
        }
        // No taking turns on the main menu
        // Also, on PlayerTurn, we already have a Label::Turn left over from RunState::AwaitingInput.
        RunState::MainMenu { .. } | RunState::GameOver | RunState::PlayerTurn => (),
        _ => {
            cae.add_effect(
                &cae.get_root(),
                Label::Turn {
                    actor: *world.player_entity(),
                },
            );
        }
    }
}
