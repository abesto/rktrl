use crate::systems::prelude::*;

cae_system_state!(DeathSystemState {
    death(link) { matches!(link.label, Label::Death {..}) }
});

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
    for death in cae.get_queue(state.death) {
        extract_label!(death @ Death => entity);

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
