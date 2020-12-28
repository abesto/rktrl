use legion::{system, systems::CommandBuffer, world::SubWorld, IntoQuery, Resources};

use crate::{
    cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link},
    components::*,
    resources::*,
};

pub struct HungerSystemState {
    subscription: CAESubscription,
}

impl HungerSystemState {
    fn subscription_filter(link: &Link) -> bool {
        matches!(link.label, Label::Turn { .. })
    }

    pub fn new(resources: &Resources) -> HungerSystemState {
        let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
        HungerSystemState {
            subscription: cae.subscribe(HungerSystemState::subscription_filter),
        }
    }
}

#[system]
#[read_component(HungerClock)]
pub fn hunger(
    #[state] state: &HungerSystemState,
    #[resource] cae: &mut CauseAndEffect,
    // TODO migrate GameLog to CAE
    #[resource] game_log: &mut GameLog,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    for turn in cae.get_queue(state.subscription) {
        let entity = match turn.label {
            Label::Turn { entity } => entity,
            _ => unreachable!(),
        };

        let clock = match <(&HungerClock,)>::query().get(world, entity) {
            Ok((clock,)) => clock,
            _ => continue,
        };
        let new_duration = clock.duration - 1;

        if new_duration >= 1 {
            commands.add_component(
                entity,
                HungerClock {
                    state: clock.state,
                    duration: new_duration,
                },
            );
        } else {
            match clock.state {
                HungerState::WellFed => {
                    cae.add_effect(&turn, Label::NoLongerWellFed);
                    commands.add_component(
                        entity,
                        HungerClock {
                            state: HungerState::Normal,
                            duration: 200,
                        },
                    );
                    game_log
                        .entries
                        .push("You are no longer well fed.".to_string());
                }
                HungerState::Normal => {
                    cae.add_effect(&turn, Label::Hungry);
                    commands.add_component(
                        entity,
                        HungerClock {
                            state: HungerState::Hungry,
                            duration: 200,
                        },
                    );
                    game_log.entries.push("You are hungry.".to_string());
                }
                HungerState::Hungry => {
                    cae.add_effect(&turn, Label::Starving);
                    commands.add_component(
                        entity,
                        HungerClock {
                            state: HungerState::Starving,
                            duration: 200,
                        },
                    );
                    game_log.entries.push("You are starving!".to_string());
                }
                HungerState::Starving => {
                    // Inflict damage from hunger
                    let hunger_pang = cae.add_effect(&turn, Label::HungerPang);
                    cae.add_effect(
                        &hunger_pang,
                        Label::Damage {
                            amount: 1,
                            to: entity,
                        },
                    );
                    game_log.entries.push(
                        "Your hunger pangs are getting painful! You suffer 1 hp damage."
                            .to_string(),
                    );
                }
            }
        }
    }
}
