use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, intents::UseIntent, name::Name, player::Player, potion::Potion,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct ItemUseSystemData<'a> {
    entity: Entities<'a>,
    name: ReadStorage<'a, Name>,
    player: ReadStorage<'a, Player>,
    wants_to_use: WriteStorage<'a, UseIntent>,
    combat_stats: WriteStorage<'a, CombatStats>,
    potion: ReadStorage<'a, Potion>,

    gamelog: WriteExpect<'a, GameLog>,
}

pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (player, to_use, stats) in (
            data.player.maybe(),
            &data.wants_to_use,
            &mut data.combat_stats,
        )
            .join()
        {
            if let Some(potion) = data.potion.get(to_use.item) {
                let new_hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                let heal_amount = new_hp - stats.hp;
                stats.hp = new_hp;
                if player.is_some() {
                    data.gamelog.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        data.name.get(to_use.item).unwrap(),
                        heal_amount
                    ));
                }
                data.entity.delete(to_use.item).expect("Delete failed");
            }
        }

        data.wants_to_use.clear();
    }
}
