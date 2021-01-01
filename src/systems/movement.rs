use legion::{system, systems::CommandBuffer, EntityStore, Resources};

use crate::cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link};
use crate::{components::*, resources::*};

pub struct MovementSystemState {
    subscription: CAESubscription,
}

impl MovementSystemState {
    fn subscription_filter(link: &Link) -> bool {
        matches!(link.label, Label::MoveIntent { .. })
    }

    pub fn new(resources: &Resources) -> MovementSystemState {
        let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
        MovementSystemState {
            subscription: cae.subscribe(MovementSystemState::subscription_filter),
        }
    }
}

#[system]
#[read_component(Position)]
pub fn movement(
    #[state] state: &MovementSystemState,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    for move_intent in cae.get_queue(state.subscription) {
        extract_label!(move_intent @ MoveIntent => target_position);
        if map.is_blocked(target_position) {
            cae.add_effect(&move_intent, Label::MovementBlocked);
            continue;
        }

        extract_nearest_ancestor!(cae, move_intent @ Turn => actor);
        commands.add_component(actor, target_position);
        cae.add_effect(&move_intent, Label::MovementDone);

        commands.exec_mut(move |w| {
            if let Ok(viewshed) = w.entry_mut(actor).unwrap().get_component_mut::<Viewshed>() {
                viewshed.dirty = true;
            }
        });
    }
}
