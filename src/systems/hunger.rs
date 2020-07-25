use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(HungerSystemData(
    entities,
    write_storage(HungerClock, SufferDamage),
    read_storage(Player),
    read(RunState),
    write(GameLog)
));

pub struct HungerSystem;

impl<'a> System<'a> for HungerSystem {
    type SystemData = HungerSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, mut clock, maybe_player) in (
            &data.entities,
            &mut data.hunger_clocks,
            data.players.maybe(),
        )
            .join()
        {
            let is_player = maybe_player.is_some();

            let proceed = match *data.run_state {
                RunState::PlayerTurn => is_player,
                RunState::MonsterTurn => !is_player,
                _ => false,
            };

            if !proceed {
                continue;
            }

            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if is_player {
                            data.game_log
                                .entries
                                .push("You are no longer well fed.".to_string());
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if is_player {
                            data.game_log.entries.push("You are hungry.".to_string());
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if is_player {
                            data.game_log.entries.push("You are starving!".to_string());
                        }
                    }
                    HungerState::Starving => {
                        // Inflict damage from hunger
                        if is_player {
                            data.game_log.entries.push(
                                "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                    .to_string(),
                            );
                        }
                        SufferDamage::new_damage(&mut data.suffer_damages, entity, 1);
                    }
                }
            }
        }
    }
}
