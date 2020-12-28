use legion::{system, systems::CommandBuffer, world::SubWorld, IntoQuery, Resources};

use crate::{
    cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link},
    components::*,
    resources::*,
};

pub struct DeathSystemState {
    subscription: CAESubscription,
}

impl DeathSystemState {
    fn subscription_filter(link: &Link) -> bool {
        matches!(link.label, Label::Death { .. })
    }

    pub fn new(resources: &Resources) -> DeathSystemState {
        let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
        DeathSystemState {
            subscription: cae.subscribe(DeathSystemState::subscription_filter),
        }
    }
}

#[system]
#[read_component(Name)]
#[read_component(Player)]
pub fn death(
    #[state] state: &DeathSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] run_state_queue: &mut RunStateQueue,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    for death in cae.get_queue(state.subscription) {
        let entity = match death.label {
            Label::Death { entity } => entity,
            _ => unreachable!(),
        };

        let (name, player) = <(&Name, Option<&Player>)>::query()
            .get(world, entity)
            .unwrap();

        if player.is_none() {
            game_log.entries.push(format!("{} is dead", name));
            commands.remove(entity)
        } else {
            run_state_queue.push_back(RunState::GameOver);
        }
    }
}
