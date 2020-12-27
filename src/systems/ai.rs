use bracket_lib::prelude::*;
use legion::{system, systems::CommandBuffer, world::SubWorld, IntoQuery};

use crate::cause_and_effect::{CauseAndEffect, Label};
use crate::util::world_ext::WorldExt;
use crate::{components::*, resources::*};

#[system]
#[read_component(Name)]
#[write_component(Viewshed)]
#[write_component(Position)]
#[write_component(Confusion)]
#[allow(clippy::too_many_arguments)]
pub fn ai(
    #[resource] game_log: &mut GameLog,
    #[resource] map: &Map,
    #[resource] cae: &mut CauseAndEffect,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut causes = cae.scan();
    while let Some(ref cause) = causes.next(cae) {
        let entity = match cause.label {
            Label::Turn { entity } => entity,
            _ => continue,
        };

        if !world.has_component::<Monster>(entity) {
            continue;
        }

        let (name, pos, viewshed, maybe_confusion) =
            <(&Name, &Position, &Viewshed, Option<&Confusion>)>::query()
                .get(world, entity)
                .unwrap();

        let can_act = {
            if let Some(confusion) = maybe_confusion {
                if let Some(new_confusion) = confusion.tick() {
                    commands.add_component(entity, new_confusion);
                } else {
                    commands.remove_component::<Confusion>(entity);
                    cae.add_effect(cause, Label::ConfusionOver { entity });
                    // TODO move game_log into an effect system
                    game_log
                        .entries
                        .push(format!("{} is no longer confused!", name));
                }
                cae.add_effect(cause, Label::SkipBecauseConfused);
                false
            } else {
                true
            }
        };

        if !can_act {
            cae.add_effect(
                cause,
                Label::ParticleRequest {
                    x: pos.x,
                    y: pos.y,
                    fg: RGB::named(MAGENTA),
                    bg: RGB::named(BLACK),
                    glyph: to_cp437('?'),
                    lifetime: 200.0,
                },
            );
            continue;
        }

        let player_pos = world.player_component::<Position>();
        let distance = DistanceAlg::Pythagoras.distance2d(**pos, *player_pos);
        if distance < 1.5 {
            cae.add_effect(cause, Label::MeleeIntent { target: player_pos });
        } else if viewshed.visible_tiles.contains(&player_pos) {
            let path = a_star_search(
                map.pos_idx(*pos) as i32,
                map.pos_idx(player_pos) as i32,
                map,
            );
            if path.success && path.steps.len() > 1 {
                cae.add_effect(
                    cause,
                    Label::MoveIntent {
                        target: map.idx_pos(path.steps[1]),
                    },
                );
            }
        } else {
            cae.add_effect(cause, Label::SkipBecauseHidden);
        }
    }
}
