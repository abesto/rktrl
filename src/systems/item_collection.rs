use legion::{system, systems::CommandBuffer, world::SubWorld, Resources};

use crate::util::world_ext::WorldExt;
use crate::{
    cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link},
    components::*,
    resources::*,
};

cae_system_state!(ItemCollectionSystemState {
    pickup_intent(link) { matches!(link.label, Label::PickupIntent {..}) }
});

#[system]
#[read_component(Name)]
#[read_component(Position)]
pub fn item_collection(
    #[state] state: &ItemCollectionSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    for cause in cae.get_queue(state.pickup_intent) {
        let actor = cae
            .extract_nearest_ancestor(&cause, |link| match link.label {
                Label::Turn { entity } => Some(entity),
                _ => None,
            })
            .unwrap();

        let pos = world.get_component::<Position>(actor);
        let target_item = map.get_tile_contents(pos).and_then(|contents| {
            contents
                .iter()
                .find(|&&entity| world.has_component::<Item>(entity))
        });

        match target_item {
            None => {
                cae.add_effect(&cause, Label::PickupNothingHere);
                game_log
                    .entries
                    .push("There is nothing here to pick up.".to_string());
            }
            Some(&item) => {
                let action = cae.add_effect(&cause, Label::PickupAction { target: item });
                commands.remove_component::<Position>(item);
                commands.add_component(item, InBackpack::new(actor));
                game_log.entries.push(format!(
                    "You pick up the {}.",
                    world.get_component::<Name>(item)
                ));
                cae.add_effect(&action, Label::PickupDone);
            }
        }
    }
}
