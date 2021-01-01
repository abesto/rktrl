use crate::systems::prelude::*;

cae_system_state!(ItemRemoveSystemState {
    remove_intent: RemoveIntent
});

#[system]
#[read_component(Name)]
pub fn item_remove(
    #[state] state: &ItemRemoveSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] cae: &mut CauseAndEffect,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    for remove_intent in cae.get_queue(state.remove_intent) {
        extract_label!(remove_intent @ RemoveIntent => item);
        extract_nearest_ancestor!(cae, remove_intent @ Turn => actor);
        commands.remove_component::<Equipped>(item);
        commands.add_component(item, InBackpack { owner: actor });
        game_log.entries.push(format!(
            "You unequip {}.",
            world.get_component::<Name>(item)
        ));
        cae.add_effect(&remove_intent, Label::RemoveDone);
    }
}
