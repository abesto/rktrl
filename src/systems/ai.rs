use crate::systems::prelude::*;

cae_system_state!(AiSystemState { subscribe(Turn) });

#[system]
#[read_component(Name)]
#[write_component(Viewshed)]
#[write_component(Position)]
#[write_component(Confusion)]
pub fn ai(
    #[state] state: &AiSystemState,
    #[resource] map: &Map,
    #[resource] cae: &mut CauseAndEffect,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    for ref cause in cae.get_queue(state.turn) {
        extract_label!(cause @ Turn => actor);

        if !world.has_component::<Monster>(actor) {
            continue;
        }

        let (pos, viewshed, maybe_confusion) =
            <(&Position, &Viewshed, Option<&Confusion>)>::query()
                .get(world, actor)
                .unwrap();

        let can_act = {
            if let Some(confusion) = maybe_confusion {
                if let Some(new_confusion) = confusion.tick() {
                    commands.add_component(actor, new_confusion);
                } else {
                    commands.remove_component::<Confusion>(actor);
                    cae.add_effect(cause, Label::ConfusionOver { entity: actor });
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
            cae.add_effect(
                cause,
                Label::MeleeIntent {
                    target_position: player_pos,
                },
            );
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
                        target_position: map.idx_pos(path.steps[1]),
                    },
                );
            }
        } else {
            cae.add_effect(cause, Label::SkipBecauseHidden);
        }
    }
}
