use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(DeathSystemData(
    entities,
    read_storage(Player, Name),
    write_storage(CombatStats),
    write(RunStateQueue, GameLog, RunState),
));

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = DeathSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, stats, name, player) in (
            &data.entities,
            &data.combat_statses,
            &data.names,
            data.players.maybe(),
        )
            .join()
        {
            if stats.hp >= 1 {
                continue;
            }
            if player.is_none() {
                data.game_log.entries.push(format!("{} is dead", name));
                data.entities.delete(entity).unwrap();
            } else {
                data.run_state_queue.push_back(RunState::GameOver);
            }
        }
    }
}
