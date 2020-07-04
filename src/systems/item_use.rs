use bracket_lib::prelude::field_of_view;
use rktrl_macros::systemdata;
use specs::prelude::*;

use crate::components::UseTarget;

systemdata!(ItemUseSystemData(
    entities
    read_storage(
        Name, Player, ProvidesHealing, InflictsDamage, Ranged, Position, AreaOfEffect, Monster
    )
    write_storage(UseIntent, CombatStats, Consumable, SufferDamage, Confusion)
    read_expect(Map)
    write_expect(GameLog)
));

pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ItemUseSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (actor_entity, actor_position, player, to_use) in (
            &data.entities,
            &data.positions,
            data.players.maybe(),
            &data.use_intents,
        )
            .join()
        {
            let mut used_item = false;
            let item_name = data.names.get(to_use.item).unwrap();

            let targets: Vec<Entity> = match to_use.target {
                UseTarget::SelfCast => vec![actor_entity],
                UseTarget::Position(target_position) => {
                    let ranged = data
                        .rangeds
                        .get(to_use.item)
                        .expect("Target specified for non-ranged item :O");
                    if (*actor_position - target_position).len() > ranged.range as f32 {
                        data.game_log
                            .entries
                            .push(format!("That's too far away for {}", item_name));
                        continue;
                    } else {
                        match data.area_of_effects.get(to_use.item) {
                            None => data
                                .map
                                .get_tile_contents(target_position)
                                .map(|x| x.to_vec())
                                .unwrap_or_default(),
                            Some(aoe) => field_of_view(*target_position, aoe.radius, &*data.map)
                                .iter()
                                .map(|p| Position::from(*p))
                                .filter(|p| data.map.contains(*p))
                                .flat_map(|p| data.map.get_tile_contents(p))
                                .flatten()
                                .cloned()
                                .collect(),
                        }
                    }
                }
            };

            used_item |= match data.provides_healings.get(to_use.item) {
                None => false,
                Some(healing) => {
                    let stats = data
                        .combat_statses
                        .get_mut(actor_entity)
                        .expect("Tried to heal an entity without combat stats");
                    let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                    let heal_amount = new_hp - stats.hp;
                    stats.hp = new_hp;
                    if player.is_some() {
                        data.game_log.entries.push(format!(
                            "You use the {}, healing {} hp.",
                            data.names.get(to_use.item).unwrap(),
                            heal_amount
                        ));
                    }
                    true
                }
            };

            used_item |= match data.inflicts_damages.get(to_use.item) {
                None => false,
                Some(damage) => {
                    let combat_targets: Vec<&Entity> = targets
                        .iter()
                        .filter(|&t| data.combat_statses.get(*t).is_some())
                        .collect();

                    if combat_targets.is_empty() {
                        false
                    } else {
                        for &target in &combat_targets {
                            SufferDamage::new_damage(
                                &mut data.suffer_damages,
                                *target,
                                damage.damage,
                            );
                            if player.is_some() {
                                let mob_name = data.names.get(*target).unwrap();
                                data.game_log.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name, mob_name, damage.damage
                                ));
                            }
                        }
                        true
                    }
                }
            };

            used_item |= match { data.confusions.get(to_use.item).cloned() } {
                None => false,
                Some(confusion) => {
                    let valid_targets: Vec<&Entity> = targets
                        .iter()
                        // TODO Allow hitting players, maybe once the AI system is generalized
                        .filter(|&t| data.monsters.get(*t).is_some())
                        .collect();

                    if valid_targets.is_empty() {
                        false
                    } else {
                        for &target in valid_targets {
                            let target_name = data
                                .names
                                .get(target)
                                .expect("Tried to confuse something with no name :O");
                            data.confusions
                                .insert(target, confusion)
                                .expect("Unable to insert Confusion");
                            data.game_log.entries.push(format!(
                                "You use {} on {}, confusing them for {} turns.",
                                item_name, target_name, confusion.turns
                            ));
                        }
                        true
                    }
                }
            };

            if used_item {
                if data.consumables.get(to_use.item).is_some() {
                    data.entities.delete(to_use.item).expect("Delete failed");
                }
            } else {
                data.game_log
                    .entries
                    .push(format!("No valid targets found for {}", item_name));
            }
        }

        data.use_intents.clear();
    }
}
