use crate::{components::*, resources::*};
use legion::{
    system,
    systems::CommandBuffer,
    world::{EntityStore, SubWorld},
    Entity,
};

#[system(for_each)]
#[read_component(Name)]
pub fn item_remove(
    actor: &Entity,
    to_remove: &RemoveIntent,
    #[resource] game_log: &mut GameLog,
    commands: &mut CommandBuffer,
    world: &SubWorld,
) {
    commands.remove_component::<Equipped>(to_remove.item);
    commands.add_component(to_remove.item, InBackpack { owner: *actor });
    game_log.entries.push(format!(
        "You unequip {}.",
        world
            .entry_ref(to_remove.item)
            .unwrap()
            .get_component::<Name>()
            .unwrap()
    ));

    commands.remove_component::<RemoveIntent>(*actor);
}
