use legion::{Entity, system, systems::CommandBuffer};

use crate::{components::*, resources::*};

#[system(for_each)]
pub fn death(
    entity: &Entity,
    stats: &mut CombatStats,
    name: &Name,
    player: Option<&Player>,
    #[resource] game_log: &mut GameLog,
    #[resource] run_state_queue: &mut RunStateQueue,
    commands: &mut CommandBuffer,
) {
    if stats.hp >= 1 {
        return;
    }
    if player.is_none() {
        game_log.entries.push(format!("{} is dead", name));
        commands.remove(*entity)
    } else {
        run_state_queue.push_back(RunState::GameOver);
    }
}
