use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        effects::{Consumable, InflictsDamage, ProvidesHealing, Ranged},
        intents::UseIntent,
        name::Name,
        player::Player,
        position::Position,
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
    ranged: ReadStorage<'a, Ranged>,
    position: ReadStorage<'a, Position>,

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
        for (actor_entity, actor_position, player, to_use) in (
            &data.entity,
            &data.position,
            data.player.maybe(),
            &data.use_intent,
        )
            .join()
        {
            let used_selfheal = if let Some(healing) = data.healing.get(to_use.item) {
                let stats = data.combat_stats.get_mut(actor_entity).unwrap();
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
                true
            } else {
                false
            };

            let used_ranged = if let Some(ranged) = data.ranged.get(to_use.item) {
                // Loop in place of early breaks from arbitrary blocks
                // https://github.com/rust-lang/rfcs/pull/2046
                #[allow(clippy::never_loop)]
                loop {
                    let target_position = to_use.target.unwrap();
                    let item_name = data.name.get(to_use.item).unwrap();

                    if (*actor_position - target_position).len() > ranged.range as f32 {
                        data.gamelog
                            .entries
                            .push(format!("That's too far away for {}", item_name));
                        break false;
                    }

                    let maybe_targets = data.map.get_tile_contents(target_position);

                    if maybe_targets.is_none() {
                        data.gamelog
                            .entries
                            .push(format!("There's nothing there for {}", item_name));
                        break false;
                    }
                    let targets = maybe_targets.unwrap();

                    let mut used_ranged_item = false;

                    if let Some(damage) = data.inflict_damage.get(to_use.item) {
                        for target in targets {
                            if data.combat_stats.get(*target).is_none() {
                                continue;
                            }
                            SufferDamage::new_damage(
                                &mut data.suffer_damage,
                                *target,
                                damage.damage,
                            );
                            if player.is_some() {
                                let mob_name = data.name.get(*target).unwrap();
                                data.gamelog.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name, mob_name, damage.damage
                                ));
                            }

                            used_ranged_item = true;
                        }
                    }

                    break used_ranged_item;
                }
            } else {
                false
            };

            let used_item = used_selfheal || used_ranged;
            if used_item && data.consumable.get(to_use.item).is_some() {
                data.entity.delete(to_use.item).expect("Delete failed");
            }
        }

        data.use_intent.clear();
    }
}
