use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        in_backpack::InBackpack, name::Name, player::Player, position::Position,
        wants_to_pick_up_item::WantsToPickUpItem,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct ItemCollectionSystemData<'a> {
    pickup: WriteStorage<'a, WantsToPickUpItem>,
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
        for (pickup, player) in (&data.pickup, data.player.maybe()).join() {
            data.position.remove(pickup.item);
            data.backpack
                .insert(pickup.item, InBackpack::new(pickup.actor))
                .expect("Unable to insert backpack entry");

            if player.is_some() {
                data.gamelog.entries.push(format!(
                    "You pick up the {}.",
                    data.name.get(pickup.item).unwrap().name
                ));
            }
        }

        data.pickup.clear();
    }
}
