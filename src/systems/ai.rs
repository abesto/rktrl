use bracket_lib::prelude::{a_star_search, DistanceAlg};
use rktrl_macros::systemdata;
use specs::prelude::*;

systemdata!(AISystemData(
    entities
    read_storage(Monster, Player, Name)
    write_storage(Viewshed, Position, MeleeIntent, Confusion)
    write_expect(GameLog)
    read(RunState)
    read_expect(Map)
));

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = AISystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if *data.run_state != RunState::MonsterTurn {
            return;
        }
        let (&player_pos, player_entity, _player) =
            (&data.positions, &data.entities, &data.players)
                .join()
                .next()
                .unwrap();
        for (entity, mut viewshed, pos, _monster) in (
            &data.entities,
            &mut data.viewsheds,
            &mut data.positions,
            &data.monsters,
        )
            .join()
        {
            let can_act = {
                if let Some(confusion) = data.confusions.get_mut(entity) {
                    confusion.turns -= 1;
                    if confusion.turns < 0 {
                        data.confusions.remove(entity);
                        data.game_log.entries.push(format!(
                            "{} is no longer confused!",
                            data.names.get(entity).unwrap()
                        ));
                    }
                    false
                } else {
                    true
                }
            };

            if !can_act {
                continue;
            }

            let distance = DistanceAlg::Pythagoras.distance2d(**pos, *player_pos);
            if distance < 1.5 {
                data.melee_intents
                    .insert(
                        entity,
                        MeleeIntent {
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
