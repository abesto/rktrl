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
    for cause in cae.get_queue(state.subscription) {
        let target = match cause.label {
            Label::MoveIntent { target } => target,
            _ => unreachable!(),
        };

        if map.is_blocked(target) {
            cae.add_effect(&cause, Label::MovementBlocked);
            continue;
        }

        let entity = match cae
            .find_first_ancestor(&cause, |ancestor| matches!(ancestor.label, Label::Turn{..}))
            .unwrap()
            .label
        {
            Label::Turn { entity } => entity,
            _ => unreachable!(),
        };
        commands.add_component(entity, target);
        cae.add_effect(&cause, Label::MovementDone);

        commands.exec_mut(move |w| {
            if let Ok(viewshed) = w.entry_mut(entity).unwrap().get_component_mut::<Viewshed>() {
                viewshed.dirty = true;
            }
        });
    }
}
