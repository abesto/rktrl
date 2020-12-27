use legion::{system, systems::CommandBuffer, EntityStore};

use crate::cause_and_effect::{CauseAndEffect, Label};
use crate::{components::*, resources::*};

#[system]
#[read_component(Position)]
pub fn movement(
    #[resource] cae: &mut CauseAndEffect,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    let mut causes = cae.scan();
    while let Some(cause) = causes.next(cae) {
        let target = match cause.label {
            Label::MoveIntent { target } => target,
            _ => continue,
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
