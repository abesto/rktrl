use crate::systems::prelude::*;

cae_system_state!(ItemUseSystemState {
    subscribe(UseIntent)
});

#[system]
#[read_component(Position)]
#[read_component(CombatStats)]
#[read_component(Ranged)]
#[read_component(AreaOfEffect)]
#[read_component(ProvidesHealing)]
#[read_component(InflictsDamage)]
#[read_component(Confusion)]
#[read_component(Monster)]
#[read_component(ProvidesFood)]
#[read_component(Consumable)]
#[read_component(Equippable)]
#[read_component(Equipped)]
#[read_component(HungerClock)]
pub fn item_use(
    #[state] state: &ItemUseSystemState,
    #[resource] map: &Map,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] deferred_cleanup: &mut DeferredCleanup,
    #[resource] run_state_queue: &mut RunStateQueue,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    for use_intent in cae.get_queue(state.use_intent) {
        extract_label!(use_intent @ UseIntent => item, target);
        extract_nearest_ancestor!(cae, use_intent @ Turn => actor);
        let (actor_position,) = <(&Position,)>::query().get(world, actor).unwrap();

        let mut used_item = false;

        let targets: &Vec<Entity> = &(match target {
            UseTarget::SelfCast => vec![actor],
            UseTarget::Position(target_position) => {
                let ranged = world.get_component::<Ranged>(item);
                if (*actor_position - target_position).len() > ranged.range as f32 {
                    cae.add_effect(&use_intent, Label::TooFarAway);
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

        for &target in targets {
            let use_on_target = cae.add_effect(&use_intent, Label::UseOnTarget { item, target });
            for f in &[
                provide_healing,
                inflict_damage,
                confuse,
                equip,
                provide_food,
            ] {
                used_item |= f(cae, world, commands, &use_on_target);
            }
            used_item |= magic_mapping(cae, &use_on_target, run_state_queue);
        }

        if used_item {
            if world.has_component::<Consumable>(item) {
                deferred_cleanup.entity(item);
            }
        } else {
            cae.add_effect(&use_intent, Label::NoValidTargets);
        }
    }
}

fn provide_food(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    _commands: &mut CommandBuffer,
    use_on_target: &Link,
) -> bool {
    extract_label!(use_on_target @ UseOnTarget => item, target);
    if !world.has_component::<ProvidesFood>(item) || !world.has_component::<HungerClock>(target) {
        return false;
    }

    cae.add_effect(
        &use_on_target,
        Label::Ate {
            who: target,
            what: item,
        },
    );
    true
}

fn equip(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_on_target: &Link,
) -> bool {
    extract_label!(use_on_target @ UseOnTarget => item, target);
    if !world.has_component::<Equippable>(item) {
        return false;
    }
    let equippable = world.get_component::<Equippable>(item);
    let target_slot = equippable.slot;

    // Remove any items the target has in the item's slot
    <(Entity, &Equipped)>::query().for_each(world, |(&already_equipped_item, already_equipped)| {
        if already_equipped.owner == target && already_equipped.slot == target_slot {
            cae.add_effect(
                &use_on_target,
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
            owner: target,
            slot: target_slot,
        },
    );
    commands.remove_component::<InBackpack>(item);
    cae.add_effect(&use_on_target, Label::EquipDone);
    true
}

fn confuse(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_on_target: &Link,
) -> bool {
    extract_label!(use_on_target @ UseOnTarget => item, target);
    // TODO Allow hitting players, maybe once the AI system is generalized
    if !world.has_component::<Confusion>(item) || !world.has_component::<Monster>(target) {
        return false;
    }
    let confusion = world.get_component::<Confusion>(item);

    let entry = world.entry_ref(target).unwrap();
    commands.add_component(target, confusion);
    cae.add_effect(&use_on_target, Label::Confused { entity: target });

    if let Ok(pos) = entry.get_component::<Position>() {
        cae.add_effect(
            &use_on_target,
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
    true
}

fn inflict_damage(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    _commands: &mut CommandBuffer,
    use_on_target: &Link,
) -> bool {
    extract_label!(use_on_target @ UseOnTarget => item, target);
    if !world.has_component::<InflictsDamage>(item) || !world.has_component::<CombatStats>(target) {
        return false;
    }
    let damage = world.get_component::<InflictsDamage>(item).damage;

    cae.add_effect(
        &use_on_target,
        Label::Damage {
            amount: damage,
            to: target,
            bleeding: true,
        },
    );

    if world.has_component::<Position>(target) {
        let pos = world.get_component::<Position>(target);
        cae.add_effect(
            &use_on_target,
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
    true
}

fn provide_healing(
    cae: &mut CauseAndEffect,
    world: &SubWorld,
    commands: &mut CommandBuffer,
    use_on_target: &Link,
) -> bool {
    extract_label!(use_on_target @ UseOnTarget => item, target);
    if !world.has_component::<ProvidesHealing>(item) || !world.has_component::<CombatStats>(target)
    {
        return false;
    }

    let healing = world.get_component::<ProvidesHealing>(item);
    let stats = world.get_component::<CombatStats>(target);
    let new_hp = i32::min(stats.max_hp, stats.hp + healing.heal_amount);
    let heal_amount = new_hp - stats.hp;

    // TODO factor this out to separate `healing` system
    //      and deal with capping the healing amount in the logged message
    commands.add_component(target, stats.with_hp(new_hp));

    cae.add_effect(
        &use_on_target,
        Label::Healing {
            to: target,
            amount: heal_amount,
        },
    );

    extract_nearest_ancestor!(cae, use_on_target @ Turn => actor);
    let actor_position = world.get_component::<Position>(actor);
    cae.add_effect(
        &use_on_target,
        Label::ParticleRequest {
            x: actor_position.x,
            y: actor_position.y,
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            glyph: to_cp437('♥'),
            lifetime: 200.0,
        },
    );

    true
}

fn magic_mapping(
    cae: &mut CauseAndEffect,
    use_on_target: &Link,
    run_state_queue: &mut RunStateQueue,
) -> bool {
    cae.add_effect(&use_on_target, Label::MagicMapping);
    run_state_queue.push_front(RunState::MagicMapReveal { row: 0 });
    true
}
