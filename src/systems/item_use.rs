use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        effects::{AreaOfEffect, Confusion, Consumable, InflictsDamage, ProvidesHealing, Ranged},
        intents::{UseIntent, UseTarget},
        monster::Monster,
        name::Name,
        player::Player,
        position::Position,
        suffer_damage::SufferDamage,
    },
    resources::{gamelog::GameLog, map::Map},
};
use bracket_lib::prelude::field_of_view;

#[derive(SystemData)]
pub struct ItemUseSystemData<'a> {
    entity: Entities<'a>,

    name: ReadStorage<'a, Name>,
    player: ReadStorage<'a, Player>,
    healing: ReadStorage<'a, ProvidesHealing>,
    inflict_damage: ReadStorage<'a, InflictsDamage>,
    ranged: ReadStorage<'a, Ranged>,
    position: ReadStorage<'a, Position>,
    area_of_effect: ReadStorage<'a, AreaOfEffect>,
    monster: ReadStorage<'a, Monster>,

    use_intent: WriteStorage<'a, UseIntent>,
    combat_stats: WriteStorage<'a, CombatStats>,
    consumable: WriteStorage<'a, Consumable>,
    suffer_damage: WriteStorage<'a, SufferDamage>,
    confusion: WriteStorage<'a, Confusion>,

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
            let mut used_item = false;
            let item_name = data.name.get(to_use.item).unwrap();

            let targets: Vec<Entity> = match to_use.target {
                UseTarget::SelfCast => vec![actor_entity],
                UseTarget::Position(target_position) => {
                    let ranged = data
                        .ranged
                        .get(to_use.item)
                        .expect("Target specified for non-ranged item :O");
                    if (*actor_position - target_position).len() > ranged.range as f32 {
                        data.gamelog
                            .entries
                            .push(format!("That's too far away for {}", item_name));
                        continue;
                    } else {
                        match data.area_of_effect.get(to_use.item) {
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

            used_item |= match data.healing.get(to_use.item) {
                None => false,
                Some(healing) => {
                    let stats = data
                        .combat_stats
                        .get_mut(actor_entity)
                        .expect("Tried to heal an entity without combat stats");
                    let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                    let heal_amount = new_hp - stats.hp;
                    stats.hp = new_hp;
                    if player.is_some() {
                        data.gamelog.entries.push(format!(
                            "You use the {}, healing {} hp.",
                            data.name.get(to_use.item).unwrap(),
                            heal_amount
                        ));
                    }
                    true
                }
            };

            used_item |= match data.inflict_damage.get(to_use.item) {
                None => false,
                Some(damage) => {
                    let combat_targets: Vec<&Entity> = targets
                        .iter()
                        .filter(|&t| data.combat_stats.get(*t).is_some())
                        .collect();

                    if combat_targets.is_empty() {
                        false
                    } else {
                        for &target in &combat_targets {
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
                        }
                        true
                    }
                }
            };

            used_item |= match { data.confusion.get(to_use.item).cloned() } {
                None => false,
                Some(confusion) => {
                    let valid_targets: Vec<&Entity> = targets
                        .iter()
                        // TODO Allow hitting players, maybe once the AI system is generalized
                        .filter(|&t| data.monster.get(*t).is_some())
                        .collect();

                    if valid_targets.is_empty() {
                        false
                    } else {
                        for &target in valid_targets {
                            let target_name = data
                                .name
                                .get(target)
                                .expect("Tried to confuse something with no name :O");
                            data.confusion
                                .insert(target, confusion)
                                .expect("Unable to insert Confusion");
                            data.gamelog.entries.push(format!(
                                "You use {} on {}, confusing them for {} turns.",
                                item_name, target_name, confusion.turns
                            ));
                        }
                        true
                    }
                }
            };

            if used_item {
                if data.consumable.get(to_use.item).is_some() {
                    data.entity.delete(to_use.item).expect("Delete failed");
                }
            } else {
                data.gamelog
                    .entries
                    .push(format!("No valid targets found for {}", item_name));
            }
        }

        data.use_intent.clear();
    }
}
