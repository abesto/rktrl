use crate::systems::prelude::*;

cae_system_state!(ItemRemoveSystemState {
    subscribe(RemoveIntent)
});

#[system]
#[read_component(Name)]
pub fn item_remove(
    #[state] state: &ItemRemoveSystemState,
    #[resource] cae: &mut CauseAndEffect,
    commands: &mut CommandBuffer,
) {
    for remove_intent in cae.get_queue(state.remove_intent) {
        extract_label!(remove_intent @ RemoveIntent => item);
        extract_nearest_ancestor!(cae, remove_intent @ Turn => actor);
        commands.remove_component::<Equipped>(item);
        commands.add_component(item, InBackpack { owner: actor });
        cae.add_effect(&remove_intent, Label::RemoveDone);
    }
}
