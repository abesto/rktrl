use crate::systems::prelude::*;

cae_system_state!(GameLogSystemState {
    ate: Ate,
    no_longer_well_fed: NoLongerWellFed,
    hungry: Hungry,
    starving: Starving,
    damage: Damage
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
    for f in &[ate, no_longer_well_fed, hungry, starving, damage] {
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
    if world.is_player(actor) {
        Some("You are no longer well fed.".to_string())
    } else {
        None
    }
});

handle_event!(hungry, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if world.is_player(actor) {
        Some("You are hungry.".to_string())
    } else {
        None
    }
});

handle_event!(starving, |state, cae, world, event| {
    extract_nearest_ancestor!(cae, event @ Turn => actor);
    if world.is_player(actor) {
        Some("You are starving!".to_string())
    } else {
        None
    }
});

handle_event!(ate, |state, cae, world, event| {
    extract_label!(event @ Ate => who, what);
    if world.is_player(who) {
        Some(format!(
            "You eat the {}.",
            world.get_component::<Name>(what)
        ))
    } else {
        None
    }
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
                None
            }
        }
        _ => None,
    }
});
