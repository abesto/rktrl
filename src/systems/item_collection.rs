use legion::{
    system,
    systems::CommandBuffer,
    world::{EntityStore, SubWorld},
    Entity,
};

use crate::{components::*, resources::*};

#[system(for_each)]
#[read_component(Name)]
pub fn item_collection(
    actor: &Entity,
    pickup: &PickupIntent,
    player: Option<&Player>,
    #[resource] game_log: &mut GameLog,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    commands.remove_component::<Position>(pickup.item);
    commands.add_component(pickup.item, InBackpack::new(*actor));

    if player.is_some() {
        game_log.entries.push(format!(
            "You pick up the {}.",
            world
                .entry_ref(pickup.item)
                .unwrap()
                .get_component::<Name>()
                .unwrap()
        ));
    }

    commands.remove_component::<PickupIntent>(*actor);
}
