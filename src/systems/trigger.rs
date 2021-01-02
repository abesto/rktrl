use crate::systems::prelude::*;

cae_system_state!(TriggerSystemState {
    subscribe(MovementDone)
});

#[system]
#[read_component(EntryTrigger)]
#[read_component(InflictsDamage)]
pub fn trigger(
    #[state] state: &TriggerSystemState,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    #[resource] deferred_cleanup: &mut DeferredCleanup,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    for movement_done in cae.get_queue(state.movement_done) {
        extract_nearest_ancestor!(cae, movement_done @ Turn => actor);
        extract_cause!(cae, movement_done @ MoveIntent => target_position);

        if let Some(entities_at_target) = map.get_tile_contents(target_position) {
            for &trigger in entities_at_target {
                if !world.has_component::<EntryTrigger>(trigger) {
                    continue;
                }
                let entry_triggered =
                    cae.add_effect(&movement_done, Label::EntryTriggered { trigger });

                if let Ok((inflicts_damage,)) = <(&InflictsDamage,)>::query().get(world, trigger) {
                    cae.add_effect(
                        &entry_triggered,
                        Label::Damage {
                            to: actor,
                            amount: inflicts_damage.damage,
                            bleeding: true,
                        },
                    );
                }

                if world.has_component::<SingleActivation>(trigger) {
                    deferred_cleanup.entity(trigger);
                } else {
                    commands.remove_component::<Hidden>(trigger);
                }
            }
        }
    }
}
