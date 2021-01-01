use crate::systems::prelude::*;

cae_system_state!(ItemCollectionSystemState {
    subscribe(PickupIntent)
});

#[system]
#[read_component(Name)]
#[read_component(Position)]
pub fn item_collection(
    #[state] state: &ItemCollectionSystemState,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    for cause in cae.get_queue(state.pickup_intent) {
        extract_nearest_ancestor!(cae, cause @ Turn => actor);

        let pos = world.get_component::<Position>(actor);
        let target_item = map.get_tile_contents(pos).and_then(|contents| {
            contents
                .iter()
                .find(|&&entity| world.has_component::<Item>(entity))
        });

        match target_item {
            None => {
                cae.add_effect(&cause, Label::PickupNothingHere);
            }
            Some(&item) => {
                let action = cae.add_effect(&cause, Label::PickupAction { item: item });
                commands.remove_component::<Position>(item);
                commands.add_component(item, InBackpack::new(actor));
                cae.add_effect(&action, Label::PickupDone);
            }
        }
    }
}
