use crate::systems::prelude::*;

cae_system_state!(GameLogSystemState {
    subscribe(
        Ate, NoLongerWellFed, Hungry, Starving,
        Damage, Death,
        ConfusionOver,
        PickupNothingHere, PickupDone, DropDone,
        RemoveDone
    )
});

#[system]
#[read_component(Player)]
#[read_component(Name)]
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
        death,
        confusion_over,
        pickup_nothing_here,
        pickup_done,
        drop_done,
        remove_done,
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
    match cae.get_cause(&damage).map(|link| link.label) {
        Some(Label::HungerPang) => {
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
    extract_nearest_ancestor!(cae, event @ PickupAction => item);
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
    extract_nearest_ancestor!(cae, event @ DropIntent => item);
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

handle_event!(remove_done, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    extract_nearest_ancestor!(cae, event @ RemoveIntent => item);
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
