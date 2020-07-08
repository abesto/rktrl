use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(ItemDropSystemData(
    entities,
    write_storage(DropIntent, Position, InBackpack),
    read_storage(Name, Player),
    write_expect(GameLog, RunState),
));

pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = ItemDropSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (actor, to_drop, player) in
            (&data.entities, &data.drop_intents, data.players.maybe()).join()
        {
            assert_eq!(
                Some(actor),
                data.in_backpacks.get(to_drop.item).map(|b| b.owner)
            );
            let position = { *data.positions.get(actor).unwrap() };
            data.positions
                .insert(to_drop.item, position)
                .expect("Unable to insert position");
            data.in_backpacks.remove(to_drop.item);

            if player.is_some() {
                data.game_log.entries.push(format!(
                    "You drop the {}.",
                    data.names.get(to_drop.item).unwrap()
                ));
            }
        }

        data.drop_intents.clear();
    }
}
