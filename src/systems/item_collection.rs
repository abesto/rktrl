use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        in_backpack::InBackpack, intents::PickupIntent, name::Name, player::Player,
        position::Position,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct ItemCollectionSystemData<'a> {
    entity: Entities<'a>,
    pickup_intent: WriteStorage<'a, PickupIntent>,
    position: WriteStorage<'a, Position>,
    name: ReadStorage<'a, Name>,
    backpack: WriteStorage<'a, InBackpack>,
    player: ReadStorage<'a, Player>,

    gamelog: WriteExpect<'a, GameLog>,
}

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ItemCollectionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (actor, pickup, player) in
            (&data.entity, &data.pickup_intent, data.player.maybe()).join()
        {
            data.position.remove(pickup.item);
            data.backpack
                .insert(pickup.item, InBackpack::new(actor))
                .expect("Unable to insert backpack entry");

            if player.is_some() {
                data.gamelog.entries.push(format!(
                    "You pick up the {}.",
                    data.name.get(pickup.item).unwrap()
                ));
            }
        }

        data.pickup_intent.clear();
    }
}
