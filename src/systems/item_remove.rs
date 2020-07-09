use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(ItemRemoveSystemData(
    entities,
    write_storage(RemoveIntent, Equipped, InBackpack),
    read_storage(Name),
    write_expect(GameLog)
));

pub struct ItemRemoveSystem;

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = ItemRemoveSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, to_remove) in (&data.entities, &data.remove_intents).join() {
            data.equippeds.remove(to_remove.item);
            data.in_backpacks
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
            data.game_log.entries.push(format!(
                "You unequip {}.",
                data.names.get(to_remove.item).unwrap()
            ));
        }

        data.remove_intents.clear();
    }
}
