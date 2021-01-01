use crate::systems::prelude::*;

cae_system_state!(HungerSystemState {
    subscribe(Turn, Ate)
});

#[system]
#[read_component(HungerClock)]
#[read_component(Name)]
pub fn hunger(
    #[state] state: &HungerSystemState,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut ate_this_turn = vec![];
    for ate in cae.get_queue(state.ate) {
        extract_label!(ate @ Ate => who);
        commands.add_component(who, HungerClock::new(HungerState::WellFed, 20));
        ate_this_turn.push(who);
    }

    for turn in cae.get_queue(state.turn) {
        extract_label!(turn @ Turn => actor);
        if ate_this_turn.contains(&actor) {
            continue;
        }

        let clock = match <(&HungerClock,)>::query().get(world, actor) {
            Ok((clock,)) => clock,
            _ => continue,
        };
        let new_duration = clock.duration - 1;

        if new_duration >= 1 {
            commands.add_component(
                actor,
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
                        actor,
                        HungerClock {
                            state: HungerState::Normal,
                            duration: 200,
                        },
                    );
                }
                HungerState::Normal => {
                    cae.add_effect(&turn, Label::Hungry);
                    commands.add_component(
                        actor,
                        HungerClock {
                            state: HungerState::Hungry,
                            duration: 200,
                        },
                    );
                }
                HungerState::Hungry => {
                    cae.add_effect(&turn, Label::Starving);
                    commands.add_component(
                        actor,
                        HungerClock {
                            state: HungerState::Starving,
                            duration: 200,
                        },
                    );
                }
                HungerState::Starving => {
                    // Inflict damage from hunger
                    let hunger_pang = cae.add_effect(&turn, Label::HungerPang);
                    cae.add_effect(
                        &hunger_pang,
                        Label::Damage {
                            amount: 1,
                            to: actor,
                            bleeding: false,
                        },
                    );
                }
            }
        }
    }
}
