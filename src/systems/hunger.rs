use legion::{system, systems::CommandBuffer, Entity};

use crate::{components::*, resources::*};

#[system(for_each)]
#[write_component(SufferDamage)]
pub fn hunger(
    entity: &Entity,
    clock: &mut HungerClock,
    maybe_player: Option<&Player>,
    #[resource] run_state: &RunState,
    #[resource] game_log: &mut GameLog,
    commands: &mut CommandBuffer,
) {
    let is_player = maybe_player.is_some();

    let proceed = match *run_state {
        RunState::PlayerTurn => is_player,
        RunState::MonsterTurn => !is_player,
        _ => false,
    };

    if !proceed {
        return;
    }

    clock.duration -= 1;
    if clock.duration < 1 {
        match clock.state {
            HungerState::WellFed => {
                clock.state = HungerState::Normal;
                clock.duration = 200;
                if is_player {
                    game_log
                        .entries
                        .push("You are no longer well fed.".to_string());
                }
            }
            HungerState::Normal => {
                clock.state = HungerState::Hungry;
                clock.duration = 200;
                if is_player {
                    game_log.entries.push("You are hungry.".to_string());
                }
            }
            HungerState::Hungry => {
                clock.state = HungerState::Starving;
                clock.duration = 200;
                if is_player {
                    game_log.entries.push("You are starving!".to_string());
                }
            }
            HungerState::Starving => {
                // Inflict damage from hunger
                if is_player {
                    game_log.entries.push(
                        "Your hunger pangs are getting painful! You suffer 1 hp damage."
                            .to_string(),
                    );
                }
                SufferDamage::new_damage(commands, *entity, 1);
            }
        }
    }
}
