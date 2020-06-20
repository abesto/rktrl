use bracket_lib::prelude::{a_star_search, DistanceAlg};
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        monster::Monster, player::Player, position::Position, viewshed::Viewshed,
        wants_to_melee::WantsToMelee,
    },
    resources::{map::Map, runstate::RunState},
};

#[derive(SystemData)]
pub struct AISystemData<'a> {
    viewshed: WriteStorage<'a, Viewshed>,
    position: WriteStorage<'a, Position>,
    monster: ReadStorage<'a, Monster>,
    player: ReadStorage<'a, Player>,
    wants_to_melee: WriteStorage<'a, WantsToMelee>,

    entities: Entities<'a>,
    runstate: Read<'a, RunState>,
    map: ReadExpect<'a, Map>,
}

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = AISystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if *data.runstate != RunState::MonsterTurn {
            return;
        }
        let (&player_pos, player_entity, _player) = (&data.position, &data.entities, &data.player)
            .join()
            .next()
            .unwrap();
        for (entity, mut viewshed, pos, _monster) in (
            &data.entities,
            &mut data.viewshed,
            &mut data.position,
            &data.monster,
        )
            .join()
        {
            let distance = DistanceAlg::Pythagoras.distance2d(**pos, *player_pos);
            if distance < 1.5 {
                data.wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
                return;
            } else if viewshed.visible_tiles.contains(&player_pos) {
                let path = a_star_search(
                    data.map.pos_idx(*pos) as i32,
                    data.map.pos_idx(player_pos) as i32,
                    &*data.map,
                );
                if path.success && path.steps.len() > 1 {
                    *pos = data.map.idx_pos(path.steps[1]);
                    viewshed.dirty = true;
                }
            }
        }
    }
}
