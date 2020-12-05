use bracket_lib::prelude::*;
use legion::{
    Entity,
    IntoQuery,
    system,
    systems::CommandBuffer, world::{EntityStore, SubWorld},
};

use crate::{
    components::*, resources::*, systems::particle::ParticleRequests, util::world_ext::WorldExt,
};

// TODO not a for_each system pending fix of https://github.com/amethyst/legion/issues/199
#[system]
// for_each components
#[read_component(Entity)]
#[read_component(Position)]
#[read_component(Player)]
#[read_component(UseIntent)]
#[read_component(CombatStats)]
// eof for_each components
#[read_component(Entity)]
#[read_component(Name)]
#[read_component(Ranged)]
#[read_component(AreaOfEffect)]
#[read_component(ProvidesHealing)]
#[read_component(InflictsDamage)]
#[read_component(Confusion)]
#[read_component(Monster)]
#[read_component(Player)]
#[read_component(ProvidesFood)]
#[read_component(Consumable)]
#[read_component(Equippable)]
#[read_component(Position)]
#[write_component(UseIntent)]
#[write_component(Equipped)]
#[write_component(HungerClock)]
#[write_component(InBackpack)]
#[allow(clippy::too_many_arguments)]
pub fn item_use(
    #[resource] game_log: &mut GameLog,
    #[resource] map: &Map,
    #[resource] particle_requests: &mut ParticleRequests,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    <(
        Entity,
        &Position,
        Option<&Player>,
        &UseIntent,
        Option<&CombatStats>,
    )>::query()
        .for_each(
            world,
            |(actor_entity, actor_position, player, to_use, maybe_stats)| {
                commands.remove_component::<UseIntent>(*actor_entity);
                let mut used_item = false;
                let item_name = world.get_component::<Name>(to_use.item).to_string();

                let targets: Vec<Entity> = match to_use.target {
                    UseTarget::SelfCast => vec![*actor_entity],
                    UseTarget::Position(target_position) => {
                        let ranged = world.get_component::<Ranged>(to_use.item);
                        if (*actor_position - target_position).len() > ranged.range as f32 {
                            game_log.push(format!("That's too far away for {}", item_name));
                            return;
                        } else if world.has_component::<AreaOfEffect>(to_use.item) {
                            let aoe = world.get_component::<AreaOfEffect>(to_use.item);
                            let positions: Vec<Position> =
                                field_of_view(*target_position, aoe.radius, map)
                                    .iter()
                                    .map(|p| Position::from(*p))
                                    .filter(|p| map.contains(*p))
                                    .collect();

                            for position in &positions {
                                particle_requests.request(
                                    position.x,
                                    position.y,
                                    RGB::named(ORANGE),
                                    RGB::named(BLACK),
                                    to_cp437('░'),
                                    200.0,
                                );
                            }

                            positions
                                .iter()
                                .flat_map(|p| map.get_tile_contents(*p))
                                .flatten()
                                .cloned()
                                .collect()
                        } else {
                            map.get_tile_contents(target_position)
                                .map(|x| x.to_vec())
                                .unwrap_or_default()
                        }
                    }
                };

                used_item |= if world.has_component::<ProvidesHealing>(to_use.item) {
                    let healing = world.get_component::<ProvidesHealing>(to_use.item);
                    let stats = maybe_stats.expect("Tried to heal an entity without combat stats");
                    let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
                    let heal_amount = new_hp - stats.hp;
                    commands.add_component(*actor_entity, stats.with_hp(new_hp));

                    if player.is_some() {
                        game_log.entries.push(format!(
                            "You use the {}, healing {} hp.",
                            item_name, heal_amount
                        ));
                    }

                    particle_requests.request(
                        actor_position.x,
                        actor_position.y,
                        RGB::named(GREEN),
                        RGB::named(BLACK),
                        to_cp437('♥'),
                        200.0,
                    );

                    true
                } else {
                    false
                };

                used_item |= if world.has_component::<InflictsDamage>(to_use.item) {
                    let damage = world.get_component::<InflictsDamage>(to_use.item).damage;
                    let combat_targets: Vec<&Entity> = targets
                        .iter()
                        .filter(|&&entity| world.has_component::<CombatStats>(entity))
                        .collect();

                    if combat_targets.is_empty() {
                        false
                    } else {
                        for &target in combat_targets {
                            SufferDamage::new_damage(commands, target, damage);

                            if player.is_some() {
                                let mob_name = world.get_component::<Name>(target);
                                game_log.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name, mob_name, damage
                                ));
                            }

                            if world.has_component::<Position>(target) {
                                let pos = world.get_component::<Position>(target);
                                particle_requests.request(
                                    pos.x,
                                    pos.y,
                                    RGB::named(RED),
                                    RGB::named(BLACK),
                                    to_cp437('‼'),
                                    200.0,
                                );
                            }
                        }
                        true
                    }
                } else {
                    false
                };

                used_item |= match {
                    world
                        .entry_ref(to_use.item)
                        .unwrap()
                        .get_component::<Confusion>()
                        .ok()
                        .cloned()
                } {
                    None => false,
                    Some(confusion) => {
                        let valid_targets: Vec<&Entity> = targets
                            .iter()
                            // TODO Allow hitting players, maybe once the AI system is generalized
                            .filter(|&t| {
                                world
                                    .entry_ref(*t)
                                    .unwrap()
                                    .get_component::<Monster>()
                                    .is_ok()
                            })
                            .collect();

                        if valid_targets.is_empty() {
                            false
                        } else {
                            for target in valid_targets {
                                let entry = world.entry_ref(*target).unwrap();
                                let target_name = entry
                                    .get_component::<Name>()
                                    .expect("Tried to confuse something with no name :O");
                                commands.add_component(*target, confusion);
                                game_log.entries.push(format!(
                                    "You use {} on {}, confusing them for {} turns.",
                                    item_name, target_name, confusion.turns
                                ));

                                if let Ok(pos) = entry.get_component::<Position>() {
                                    particle_requests.request(
                                        pos.x,
                                        pos.y,
                                        RGB::named(MAGENTA),
                                        RGB::named(BLACK),
                                        to_cp437('?'),
                                        200.0,
                                    );
                                }
                            }
                            true
                        }
                    }
                };

                used_item |= match {
                    world
                        .entry_ref(to_use.item)
                        .unwrap()
                        .get_component::<Equippable>()
                        .ok()
                        .cloned()
                } {
                    None => false,
                    Some(equippable) => {
                        let target_slot = equippable.slot;
                        let target = &targets[0];
                        let target_entry = world.entry_ref(*target).unwrap();

                        // Remove any items the target has in the item's slot
                        let mut to_unequip: Vec<Entity> = Vec::new();
                        <(Entity, &Equipped, &Name)>::query().for_each(
                            world,
                            |(item_entity, already_equipped, name)| {
                                if already_equipped.owner == *target
                                    && already_equipped.slot == target_slot
                                {
                                    to_unequip.push(*item_entity);
                                    if target_entry.get_component::<Player>().is_ok() {
                                        game_log.entries.push(format!("You unequip {}.", name));
                                    }
                                }
                            },
                        );
                        for item in to_unequip.iter() {
                            commands.remove_component::<Equipped>(*item);
                            commands.add_component(*item, InBackpack::new(*target));
                        }

                        // Wield the item
                        commands.add_component(
                            to_use.item,
                            Equipped {
                                owner: *target,
                                slot: target_slot,
                            },
                        );
                        commands.remove_component::<InBackpack>(to_use.item);
                        if target_entry.archetype().layout().has_component::<Player>() {
                            game_log.entries.push(format!("You equip {}.", item_name));
                        }

                        true
                    }
                };

                used_item |= if world.has_component::<ProvidesFood>(to_use.item) {
                    let &target = &targets[0];
                    if world.has_component::<HungerClock>(target) {
                        commands.add_component(target, HungerClock::new(HungerState::WellFed, 20));
                        game_log.entries.push(format!("You eat the {}.", item_name));
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if used_item {
                    if world.has_component::<Consumable>(to_use.item) {
                        commands.remove(to_use.item);
                    }
                } else {
                    game_log
                        .entries
                        .push(format!("No valid targets found for {}", item_name));
                }
            },
        );
}
