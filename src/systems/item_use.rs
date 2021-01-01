use crate::systems::prelude::*;

cae_system_state!(ItemUseSystemState {
    use_intent(link) { matches!(link.label, Label::UseIntent {..}) }
});

#[system]
#[read_component(Position)]
#[read_component(Player)]
#[read_component(CombatStats)]
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
#[write_component(Equipped)]
#[write_component(HungerClock)]
#[write_component(InBackpack)]
#[allow(clippy::too_many_arguments)]
pub fn item_use(
    #[state] state: &ItemUseSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] map: &Map,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    for use_intent in cae.get_queue(state.use_intent) {
        extract_label!(use_intent @ UseIntent => item, target);
        extract_nearest_ancestor!(cae, use_intent @ Turn => actor);
        let (actor_position,) = <(&Position,)>::query().get(world, actor).unwrap();

        let mut used_item = false;
        let item_name = world.get_component::<Name>(item).to_string();

        let targets: &Vec<Entity> = &(match target {
            UseTarget::SelfCast => vec![actor],
            UseTarget::Position(target_position) => {
                let ranged = world.get_component::<Ranged>(item);
                if (*actor_position - target_position).len() > ranged.range as f32 {
                    cae.add_effect(&use_intent, Label::TooFarAway);
                    game_log.push(format!("That's too far away for {}", item_name));
                    continue;
                } else if world.has_component::<AreaOfEffect>(item) {
                    let aoe = world.get_component::<AreaOfEffect>(item);
                    let positions: Vec<Position> = field_of_view(*target_position, aoe.radius, map)
                        .iter()
                        .map(|p| Position::from(*p))
                        .filter(|p| map.contains(*p))
                        .collect();

                    for position in &positions {
                        cae.add_effect(
                            &use_intent,
                            Label::ParticleRequest {
                                x: position.x,
                                y: position.y,
                                fg: RGB::named(ORANGE),
                                bg: RGB::named(BLACK),
                                glyph: to_cp437('░'),
                                lifetime: 200.0,
                            },
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
        });

        used_item |= provide_healing(
            game_log,
            cae,
            world,
            commands,
            &use_intent,
            item,
            actor_position,
            &item_name,
            targets,
        );
        used_item |= inflict_damage(game_log, cae, world, &use_intent, item, &item_name, targets);
        used_item |= confuse(
            game_log,
            cae,
            world,
            commands,
            &use_intent,
            item,
            &item_name,
            targets,
        );
        used_item |= equip(
            game_log,
            cae,
            world,
            commands,
            &use_intent,
            item,
            &item_name,
            &targets,
        );
        used_item |= provide_food(cae, world, &use_intent, item, &targets);

        if used_item {
            if world.has_component::<Consumable>(item) {
                // TODO we can't fully remove the entity here because downstream systems then
                //      couldn't read its components. Might need a deferred cleanup system,
                //      separate from schedule.flush
                commands.remove_component::<InBackpack>(item);
                commands.remove_component::<Position>(item);
                //commands.remove(item);
            }
        } else {
            cae.add_effect(&use_intent, Label::NoValidTargets);
            game_log
                .entries
                .push(format!("No valid targets found for {}", item_name));
        }
    }
}

fn provide_food(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    use_intent: &Link,
    item: Entity,
    targets: &&Vec<Entity>,
) -> bool {
    if !world.has_component::<ProvidesFood>(item) {
        return false;
    }

    let &target = &targets[0];
    if !world.has_component::<HungerClock>(target) {
        return false;
    }

    cae.add_effect(
        &use_intent,
        Label::Ate {
            who: target,
            what: item,
        },
    );
    true
}

fn equip(
    game_log: &mut GameLog,
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_intent: &Link,
    item: Entity,
    item_name: &String,
    targets: &&Vec<Entity>,
) -> bool {
    if !world.has_component::<Equippable>(item) {
        return false;
    }
    let equippable = world.get_component::<Equippable>(item);
    let target_slot = equippable.slot;
    let target = &targets[0];

    // Remove any items the target has in the item's slot
    <(Entity, &Equipped)>::query().for_each(world, |(&already_equipped_item, already_equipped)| {
        if already_equipped.owner == *target && already_equipped.slot == target_slot {
            cae.add_effect(
                &use_intent,
                Label::RemoveIntent {
                    item: already_equipped_item,
                },
            );
        }
    });

    // Wield the item
    commands.add_component(
        item,
        Equipped {
            owner: *target,
            slot: target_slot,
        },
    );
    commands.remove_component::<InBackpack>(item);
    cae.add_effect(&use_intent, Label::EquipDone);
    // TODO ensure remove is handled before equip in game_log system
    game_log.entries.push(format!("You equip {}.", item_name));

    true
}

fn confuse(
    game_log: &mut GameLog,
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_intent: &Link,
    item: Entity,
    item_name: &String,
    targets: &Vec<Entity>,
) -> bool {
    if !world.has_component::<Confusion>(item) {
        return false;
    }
    let confusion = world.get_component::<Confusion>(item);

    let valid_targets: Vec<&Entity> = targets
        .iter()
        // TODO Allow hitting players, maybe once the AI system is generalized
        .filter(|&&t| world.has_component::<Monster>(t))
        .collect();
    if valid_targets.is_empty() {
        return false;
    }

    for target in valid_targets {
        let entry = world.entry_ref(*target).unwrap();
        let target_name = entry
            .get_component::<Name>()
            .expect("Tried to confuse something with no name :O");
        commands.add_component(*target, confusion);
        cae.add_effect(&use_intent, Label::Confused { entity: *target });
        game_log.entries.push(format!(
            "You use {} on {}, confusing them for {} turns.",
            item_name, target_name, confusion.turns
        ));

        if let Ok(pos) = entry.get_component::<Position>() {
            cae.add_effect(
                &use_intent,
                Label::ParticleRequest {
                    x: pos.x,
                    y: pos.y,
                    fg: RGB::named(MAGENTA),
                    bg: RGB::named(BLACK),
                    glyph: to_cp437('?'),
                    lifetime: 200.0,
                },
            );
        }
    }
    true
}

fn inflict_damage(
    game_log: &mut GameLog,
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    use_intent: &Link,
    item: Entity,
    item_name: &String,
    targets: &Vec<Entity>,
) -> bool {
    if !world.has_component::<InflictsDamage>(item) {
        return false;
    }
    let damage = world.get_component::<InflictsDamage>(item).damage;

    let combat_targets: Vec<&Entity> = targets
        .iter()
        .filter(|&&entity| world.has_component::<CombatStats>(entity))
        .collect();
    if combat_targets.is_empty() {
        return false;
    }

    for &target in combat_targets {
        cae.add_effect(
            &use_intent,
            Label::Damage {
                amount: damage,
                to: target,
                bleeding: true,
            },
        );

        // TODO move into game_log system
        let mob_name = world.get_component::<Name>(target);
        game_log.entries.push(format!(
            "You use {} on {}, inflicting {} hp.",
            item_name, mob_name, damage
        ));

        if world.has_component::<Position>(target) {
            let pos = world.get_component::<Position>(target);
            cae.add_effect(
                &use_intent,
                Label::ParticleRequest {
                    x: pos.x,
                    y: pos.y,
                    fg: RGB::named(RED),
                    bg: RGB::named(BLACK),
                    glyph: to_cp437('‼'),
                    lifetime: 200.0,
                },
            );
        }
    }
    true
}

fn provide_healing(
    game_log: &mut GameLog,
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_intent: &Link,
    item: Entity,
    actor_position: &Position,
    item_name: &String,
    targets: &Vec<Entity>,
) -> bool {
    if !world.has_component::<ProvidesHealing>(item) {
        return false;
    }
    let healing = world.get_component::<ProvidesHealing>(item);

    for &target in targets {
        let stats = world.get_component::<CombatStats>(target);
        let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
        let heal_amount = new_hp - stats.hp;

        // TODO factor this out to separate `healing` system
        //      and deal with capping the healing amount in the logged message
        commands.add_component(target, stats.with_hp(new_hp));

        game_log.entries.push(format!(
            "You use the {}, healing {} hp.",
            item_name, heal_amount
        ));

        cae.add_effect(
            &use_intent,
            Label::Healing {
                to: target,
                amount: heal_amount,
            },
        );
        cae.add_effect(
            &use_intent,
            Label::ParticleRequest {
                x: actor_position.x,
                y: actor_position.y,
                fg: RGB::named(GREEN),
                bg: RGB::named(BLACK),
                glyph: to_cp437('♥'),
                lifetime: 200.0,
            },
        );
    }

    true
}
