use crate::systems::prelude::*;
use legion::EntityStore;

cae_system_state!(ItemDropSystemState {
    drop_intent(link) { matches!(link.label, Label::DropIntent {..}) }
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
        extract_label!(intent @ DropIntent => target);
        extract_nearest_ancestor!(cae, intent @ Turn => entity);
        let position = world.get_component::<Position>(entity);

        let to_drop = world.entry_ref(target).unwrap();
        assert_eq!(
            Ok(entity),
            to_drop.get_component::<InBackpack>().map(|b| b.owner)
        );
        commands.add_component(target, position);
        commands.remove_component::<InBackpack>(target);

        game_log.entries.push(format!(
            "You drop the {}.",
            to_drop.get_component::<Name>().unwrap()
        ));
        commands.remove_component::<DropIntent>(entity);
    }
}
