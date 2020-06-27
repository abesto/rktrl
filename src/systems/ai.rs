use bracket_lib::prelude::{a_star_search, DistanceAlg};
use shipyard::*;

use crate::{
    components::{
        intents::MeleeIntent, monster::Monster, player::Player, position::Position,
        viewshed::Viewshed,
    },
    resources::{map::Map, runstate::RunState},
};

pub fn ai(
    entities: EntitiesView,

    players: View<Player>,
    monsters: View<Monster>,

    mut melee_intents: ViewMut<MeleeIntent>,
    mut positions: ViewMut<Position>,
    mut viewsheds: ViewMut<Viewshed>,

    runstate: UniqueView<RunState>,
    map: UniqueView<Map>,
) {
    if *runstate != RunState::MonsterTurn {
        return;
    }
    let (player_entity, (&player_pos, _player)) =
        (&positions, &players).iter().with_id().next().unwrap();
    for (entity, (viewshed, pos, _monster)) in
        (&mut viewsheds, &mut positions, &monsters).iter().with_id()
    {
        let distance = DistanceAlg::Pythagoras.distance2d(**pos, *player_pos);
        if distance < 1.5 {
            entities.add_component(
                &mut melee_intents,
                MeleeIntent {
                    target: player_entity,
                },
                entity,
            );
            return;
        } else if viewshed.visible_tiles.contains(&player_pos) {
            let path = a_star_search(
                map.pos_idx(*pos) as i32,
                map.pos_idx(player_pos) as i32,
                &*map,
            );
            if path.success && path.steps.len() > 1 {
                *pos = map.idx_pos(path.steps[1]);
                viewshed.dirty = true;
            }
        }
    }
}
