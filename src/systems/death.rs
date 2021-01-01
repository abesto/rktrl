use crate::systems::prelude::*;

cae_system_state!(DeathSystemState { subscribe(Death) });

#[system]
#[read_component(Name)]
#[read_component(Player)]
pub fn death(
    #[state] state: &DeathSystemState,
    #[resource] run_state_queue: &mut RunStateQueue,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] deferred_cleanup: &mut DeferredCleanup,
    world: &SubWorld,
) {
    for death in cae.get_queue(state.death) {
        extract_label!(death @ Death => entity);
        if world.is_player(entity) {
            run_state_queue.push_back(RunState::GameOver);
        } else {
            deferred_cleanup.entity(entity);
        }
    }
}
