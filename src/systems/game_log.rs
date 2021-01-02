use crate::systems::prelude::*;

cae_system_state!(GameLogSystemState {
    subscribe(
        Ate, NoLongerWellFed, Hungry, Starving,
        Damage, Healing, Death,
        Confused, ConfusionOver,
        PickupNothingHere, PickupDone, DropDone,
        EquipDone, RemoveDone, NoValidTargets, TooFarAway,
        NoStairsHere, MovedToNextLevel,
        MagicMapping, Spotted,
        EntryTriggered,
    )
});

#[system]
#[read_component(Player)]
#[read_component(Name)]
#[read_component(Confusion)]
pub fn game_log(
    #[state] state: &GameLogSystemState,
    #[resource] game_log: &mut GameLog,
    #[resource] cae: &mut CauseAndEffect,
    world: &SubWorld,
) {
    for f in &[
        ate,
        no_longer_well_fed,
        hungry,
        starving,
        damage,
        healing,
        death,
        confused,
        confusion_over,
        pickup_nothing_here,
        pickup_done,
        drop_done,
        remove_done,
        equip_done,
        too_far_away,
        no_valid_targets,
        no_stairs_here,
        moved_to_next_level,
        magic_mapping,
        entry_triggered,
        spotted,
    ] {
        for msg in f(state, cae, world) {
            game_log.push(msg);
        }
    }
}

macro_rules! handle_event {
    ($queue:ident, |$state:ident, $cae:ident, $world:ident, $event:ident| $body:expr ) => {
        fn $queue(
            $state: &GameLogSystemState,
            $cae: &mut CauseAndEffect,
            $world: &SubWorld,
        ) -> Vec<String> {
            $cae.get_queue($state.$queue)
                .iter()
                .flat_map(|$event| $body)
                .collect()
        }
    };
}

handle_event!(no_longer_well_fed, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if !world.is_player(actor) {
        return None;
    }
    Some("You are no longer well fed.".to_string())
});

handle_event!(hungry, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if !world.is_player(actor) {
        return None;
    }
    Some("You are hungry.".to_string())
});

handle_event!(starving, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if !world.is_player(actor) {
        return None;
    }
    Some("You are starving!".to_string())
});

handle_event!(ate, |state, cae, world, event| {
    extract_label!(event @ Ate => who, what);
    if !world.is_player(who) {
        return None;
    }
    Some(format!(
        "You eat the {}.",
        world.get_component::<Name>(what)
    ))
});

handle_event!(damage, |state, cae, world, damage| {
    extract_label!(damage @ Damage => to, amount);
    extract_nearest_ancestor!(cae, damage @ Turn => actor);
    match cae.get_cause(&damage).map(|link| link.label).unwrap() {
        Label::HungerPang => {
            if world.is_player(to) {
                Some(format!(
                    "Your hunger pangs are getting painful! You suffer {} hp damage.",
                    amount
                ))
            } else {
                Some(format!(
                    "The stomach of {} grumbles loudly.",
                    world.get_component::<Name>(to)
                ))
            }
        }
        Label::UseOnTarget {
            item,
            target: use_target,
        } => {
            assert_eq!(use_target, to);
            assert!(world.is_player(actor));
            Some(format!(
                "You use {} on {}, inflicting {} hp of damage.",
                world.get_component::<Name>(item),
                world.get_component::<Name>(to),
                amount
            ))
        }
        Label::Hit => {
            let actor_name = world.get_component::<Name>(actor);
            let target_name = if world.is_player(to) {
                "you".to_string()
            } else {
                world.get_component::<Name>(to).into()
            };
            if world.is_player(actor) {
                if amount <= 0 {
                    Some(format!("You are unable to hurt {}.", target_name))
                } else {
                    Some(format!("You hit {}, for {} hp.", target_name, amount))
                }
            } else if amount <= 0 {
                Some(format!("{} is unable to hurt {}.", actor_name, target_name))
            } else {
                Some(format!(
                    "{} hits {}, for {} hp.",
                    actor_name, target_name, amount
                ))
            }
        }
        Label::EntryTriggered { trigger } => {
            let (actor_name, trigger_verb) = if world.is_player(actor) {
                ("You".to_string(), "trigger".to_string())
            } else {
                (
                    world.get_component::<Name>(actor).into(),
                    "triggers".to_string(),
                )
            };
            Some(format!(
                "{} {} {}, suffering {} hp damage.",
                actor_name,
                trigger_verb,
                world.get_component::<Name>(trigger),
                amount
            ))
        }
        _ => None,
    }
});

handle_event!(confusion_over, |state, cae, world, event| {
    extract_label!(event @ ConfusionOver => entity);
    assert!(!world.is_player(entity));
    Some(format!(
        "{} is no longer confused!",
        world.get_component::<Name>(entity)
    ))
});

handle_event!(pickup_nothing_here, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if !world.is_player(actor) {
        return None;
    }
    Some("There is nothing here to pick up.".to_string())
});

handle_event!(pickup_done, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_cause!(cae, event @ PickupAction => item);
    let item_name = world.get_component::<Name>(item);

    Some(if world.is_player(actor) {
        format!("You pick up the {}.", item_name)
    } else {
        format!(
            "The {} picks up the {}.",
            world.get_component::<Name>(actor),
            item_name
        )
    })
});

handle_event!(drop_done, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_cause!(cae, event @ DropIntent => item);
    let item_name = world.get_component::<Name>(item);

    Some(if world.is_player(actor) {
        format!("You drop the {}.", item_name)
    } else {
        format!(
            "The {} drops the {}.",
            world.get_component::<Name>(actor),
            item_name
        )
    })
});

handle_event!(equip_done, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_cause!(cae, event @ UseOnTarget => item, target);
    assert_eq!(actor, target); // This may be removed to allow advanced reverse pickpocketing I guess
    let item_name = world.get_component::<Name>(item);

    Some(if world.is_player(actor) {
        format!("You equip {}.", item_name)
    } else {
        format!(
            "The {} equips {}.",
            world.get_component::<Name>(actor),
            item_name
        )
    })
});

handle_event!(remove_done, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_cause!(cae, event @ RemoveIntent => item);
    let item_name = world.get_component::<Name>(item);

    Some(if world.is_player(actor) {
        format!("You unequip {}.", item_name)
    } else {
        format!(
            "The {} unequips {}.",
            world.get_component::<Name>(actor),
            item_name
        )
    })
});

handle_event!(death, |state, cae, world, event| {
    extract_label!(event @ Death => entity);
    if world.is_player(entity) {
        return None;
    }
    Some(format!("{} is dead.", world.get_component::<Name>(entity)))
});

handle_event!(too_far_away, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_nearest_ancestor!(cae, event @ UseIntent => item);
    if !world.is_player(actor) {
        return None;
    }
    Some(format!(
        "That's too far away for {}.",
        world.get_component::<Name>(item)
    ))
});

handle_event!(no_valid_targets, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_nearest_ancestor!(cae, event @ UseIntent => item);
    if !world.is_player(actor) {
        return None;
    }
    Some(format!(
        "No valid targets found for {}.",
        world.get_component::<Name>(item)
    ))
});

handle_event!(healing, |state, cae, world, event| {
    extract_label!(event @ Healing => amount, to);
    extract_cause!(cae, event @ UseOnTarget => item, target);
    assert!(world.is_player(to));
    assert_eq!(to, target);
    Some(format!(
        "You use {}, healing {} hp.",
        world.get_component::<Name>(item),
        amount
    ))
});

handle_event!(confused, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_cause!(cae, event @ UseOnTarget => item, target);
    extract_label!(event @ Confused => entity);
    assert!(world.is_player(actor));
    assert_eq!(target, entity);

    Some(format!(
        "You use {} on {}, confusing them for {} turns.",
        world.get_component::<Name>(item),
        world.get_component::<Name>(entity),
        world.get_component::<Confusion>(entity).turns
    ))
});

handle_event!(no_stairs_here, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    assert!(world.is_player(actor));
    Some("There is no way down from here.".to_string())
});

handle_event!(moved_to_next_level, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    assert!(world.is_player(actor));
    Some("You descend to the next level, and take a moment to heal.".to_string())
});

handle_event!(magic_mapping, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    assert!(world.is_player(actor));
    Some("The map is revealed to you!".to_string())
});

handle_event!(spotted, |state, cae, world, event| {
    extract_label!(event @ Spotted => hidden);
    Some(format!(
        "You spotted a {}.",
        world.get_component::<Name>(hidden)
    ))
});

handle_event!(entry_triggered, |state, cae, world, event| {
    // If the trigger causes damage, it'll be handled as part of the damage event.
    // TODO if this becomes a repeating pattern, it may be better to create machinery
    //      to handle each node only once
    if cae.has_effect(event, |link| matches!(link.label, Label::Damage {..})) {
        return None;
    }
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_label!(event @ EntryTriggered => trigger);
    Some(format!(
        "{} triggers {}!",
        world.get_component::<Name>(actor),
        world.get_component::<Name>(trigger)
    ))
});
