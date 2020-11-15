use legion::{
    system,
    systems::CommandBuffer,
    world::{EntityStore, SubWorld},
    Entity,
};

use crate::{components::*, resources::*};

#[system(for_each)]
#[read_component(Name)]
#[write_component(InBackpack)]
pub fn item_drop(
    actor: &Entity,
    drop_intent: &DropIntent,
    position: &Position,
    player: Option<&Player>,
    #[resource] game_log: &mut GameLog,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let to_drop = world.entry_ref(drop_intent.item).unwrap();
    assert_eq!(
        Ok(*actor),
        to_drop.get_component::<InBackpack>().map(|b| b.owner)
    );
    commands.add_component(drop_intent.item, *position);
    commands.remove_component::<InBackpack>(drop_intent.item);

    if player.is_some() {
        game_log.entries.push(format!(
            "You drop the {}.",
            to_drop.get_component::<Name>().unwrap()
        ));
    }
    commands.remove_component::<DropIntent>(*actor);
}
