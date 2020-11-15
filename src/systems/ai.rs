use bracket_lib::prelude::*;
use legion::{
    query::component, system, systems::CommandBuffer, world::SubWorld, Entity, IntoQuery,
};

use crate::{components::*, resources::*, systems::particle::ParticleRequests};

// TODO This should be a for_each system, but cannot currently be due to
// https://github.com/amethyst/legion/issues/199
#[system]
// for_each components
#[read_component(Entity)]
#[read_component(Name)]
#[write_component(Viewshed)]
#[write_component(Position)]
#[write_component(Confusion)]
#[filter(component::<Monster>())]
// eof for_each components
#[read_component(Player)]
#[allow(clippy::too_many_arguments)]
pub fn ai(
    #[resource] run_state: &RunState,
    #[resource] game_log: &mut GameLog,
    #[resource] particle_requests: &mut ParticleRequests,
    #[resource] map: &Map,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if *run_state != RunState::MonsterTurn {
        return;
    }

    let (&mut player_pos, &player_entity) = <(&mut Position, Entity)>::query()
        .filter(component::<Player>())
        .iter_mut(world)
        .next()
        .unwrap();

    <(
        Entity,
        &Name,
        &mut Viewshed,
        &mut Position,
        Option<&mut Confusion>,
    )>::query()
    .filter(component::<Monster>())
    .for_each_mut(world, |(entity, name, viewshed, pos, maybe_confusion)| {
        let can_act = {
            if let Some(confusion) = maybe_confusion {
                confusion.turns -= 1;
                if confusion.turns < 0 {
                    commands.remove_component::<Confusion>(*entity);
                    game_log
                        .entries
                        .push(format!("{} is no longer confused!", name));
                }
                false
            } else {
                true
            }
        };

        if !can_act {
            particle_requests.request(
                pos.x,
                pos.y,
                RGB::named(MAGENTA),
                RGB::named(BLACK),
                to_cp437('?'),
                200.0,
            );
            return;
        }

        let distance = DistanceAlg::Pythagoras.distance2d(**pos, *player_pos);
        if distance < 1.5 {
            commands.add_component(
                *entity,
                MeleeIntent {
                    target: player_entity,
                },
            );
        } else if viewshed.visible_tiles.contains(&player_pos) {
            let path = a_star_search(
                map.pos_idx(*pos) as i32,
                map.pos_idx(player_pos) as i32,
                map,
            );
            if path.success && path.steps.len() > 1 {
                *pos = map.idx_pos(path.steps[1]);
                viewshed.dirty = true;
            }
        }
    });
}
