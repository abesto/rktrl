use rktrl_macros::systemdata;
use specs::prelude::*;

use crate::{components::*, resources::*};

systemdata!(ItemCollectionSystemData(
    entities,
    read_storage(Name, Player),
    write_storage(PickupIntent, Position, InBackpack),
    write_expect(GameLog)
));

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ItemCollectionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (actor, pickup, player) in
            (&data.entities, &data.pickup_intents, data.players.maybe()).join()
        {
            data.positions.remove(pickup.item);
            data.in_backpacks
                .insert(pickup.item, InBackpack::new(actor))
                .expect("Unable to insert backpack entry");

            if player.is_some() {
                data.game_log.entries.push(format!(
                    "You pick up the {}.",
                    data.names.get(pickup.item).unwrap()
                ));
            }
        }

        data.pickup_intents.clear();
    }
}
