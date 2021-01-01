use crate::systems::prelude::*;
use legion::EntityStore;

cae_system_state!(ItemDropSystemState {
    drop_intent: DropIntent
});

#[system]
#[read_component(Name)]
#[read_component(InBackpack)]
#[read_component(Position)]
pub fn item_drop(
    #[state] state: &ItemDropSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    for intent in cae.get_queue(state.drop_intent) {
        extract_label!(intent @ DropIntent => item);
        extract_nearest_ancestor!(cae, intent @ Turn => actor);
        let position = world.get_component::<Position>(actor);

        let to_drop = world.entry_ref(item).unwrap();
        assert_eq!(
            Ok(actor),
            to_drop.get_component::<InBackpack>().map(|b| b.owner)
        );
        commands.add_component(item, position);
        commands.remove_component::<InBackpack>(item);

        game_log.entries.push(format!(
            "You drop the {}.",
            to_drop.get_component::<Name>().unwrap()
        ));
        cae.add_effect(&intent, Label::DropDone);
    }
}
