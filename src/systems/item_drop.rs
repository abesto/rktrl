use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        in_backpack::InBackpack, intents::DropIntent, name::Name, player::Player,
        position::Position,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct ItemDropSystemData<'a> {
    entities: Entities<'a>,
    drop_intent: WriteStorage<'a, DropIntent>,
    position: WriteStorage<'a, Position>,
    name: ReadStorage<'a, Name>,
    backpack: WriteStorage<'a, InBackpack>,
    player: ReadStorage<'a, Player>,

    gamelog: WriteExpect<'a, GameLog>,
}

pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = ItemDropSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (actor, to_drop, player) in
        (&data.entities, &data.drop_intent, data.player.maybe()).join()
        {
            assert_eq!(
                Some(actor),
                data.backpack.get(to_drop.item).map(|b| b.owner)
            );
            let position = { *data.position.get(actor).unwrap() };
            data.position
                .insert(to_drop.item, position)
                .expect("Unable to insert position");
            data.backpack.remove(to_drop.item);

            if player.is_some() {
                data.gamelog.entries.push(format!(
                    "You drop the {}.",
                    data.name.get(to_drop.item).unwrap()
                ));
            }
        }

        data.drop_intent.clear();
    }
}
