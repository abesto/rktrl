use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        effects::{Consumable, InflictsDamage, ProvidesHealing},
        intents::UseIntent,
        name::Name,
        player::Player,
        suffer_damage::SufferDamage,
    },
    resources::{gamelog::GameLog, map::Map},
};

#[derive(SystemData)]
pub struct ItemUseSystemData<'a> {
    entity: Entities<'a>,

    name: ReadStorage<'a, Name>,
    player: ReadStorage<'a, Player>,
    healing: ReadStorage<'a, ProvidesHealing>,
    inflict_damage: ReadStorage<'a, InflictsDamage>,

    use_intent: WriteStorage<'a, UseIntent>,
    combat_stats: WriteStorage<'a, CombatStats>,
    consumable: WriteStorage<'a, Consumable>,
    suffer_damage: WriteStorage<'a, SufferDamage>,

    map: ReadExpect<'a, Map>,

    gamelog: WriteExpect<'a, GameLog>,
}

pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (player_entity, player, to_use) in
            (&data.entity, data.player.maybe(), &data.use_intent).join()
        {
            let mut used_item = false;

            if let Some(healing) = data.healing.get(to_use.item) {
                let stats = data.combat_stats.get_mut(player_entity).unwrap();
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
                used_item = true;
            }

            if let Some(damage) = data.inflict_damage.get(to_use.item) {
                let target_position = to_use.target.unwrap();
                let item_name = data.name.get(to_use.item).unwrap();
                if let Some(target_candidates) = data.map.get_tile_contents(target_position) {
                    for target in target_candidates {
                        if data.combat_stats.get(*target).is_none() {
                            continue;
                        }
                        SufferDamage::new_damage(&mut data.suffer_damage, *target, damage.damage);
                        if player.is_some() {
                            let mob_name = data.name.get(*target).unwrap();
                            data.gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name, mob_name, damage.damage
                            ));
                        }

                        used_item = true;
                    }
                }
                if !used_item {
                    data.gamelog
                        .entries
                        .push(format!("Invalid target for {}", item_name));
                }
            }

            if used_item && data.consumable.get(to_use.item).is_some() {
                data.entity.delete(to_use.item).expect("Delete failed");
            }
        }

        data.use_intent.clear();
    }
}
