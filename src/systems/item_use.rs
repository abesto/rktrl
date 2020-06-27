use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        effects::{Consumable, ProvidesHealing},
        intents::UseIntent,
        name::Name,
        player::Player,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct ItemUseSystemData<'a> {
    entity: Entities<'a>,

    name: ReadStorage<'a, Name>,
    player: ReadStorage<'a, Player>,
    healing: ReadStorage<'a, ProvidesHealing>,

    use_intent: WriteStorage<'a, UseIntent>,
    combat_stats: WriteStorage<'a, CombatStats>,
    consumable: WriteStorage<'a, Consumable>,

    gamelog: WriteExpect<'a, GameLog>,
}

pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (player, to_use, stats) in (
            data.player.maybe(),
            &data.use_intent,
            &mut data.combat_stats,
        )
            .join()
        {
            if let Some(healing) = data.healing.get(to_use.item) {
                let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                let heal_amount = new_hp - stats.hp;
                stats.hp = new_hp;
                if player.is_some() {
                    data.gamelog.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        data.name.get(to_use.item).unwrap(),
                        heal_amount
                    ));
                }
            }

            if data.consumable.get(to_use.item).is_some() {
                data.entity.delete(to_use.item).expect("Delete failed");
            }
        }

        data.use_intent.clear();
    }
}
