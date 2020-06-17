use bracket_lib::prelude::{a_star_search, console};
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        monster::Monster, name::Name, player::Player, position::Position, viewshed::Viewshed,
    },
    resources::{map::Map, runstate::RunState},
};

#[derive(SystemData)]
pub struct AISystemData<'a> {
    viewshed: WriteStorage<'a, Viewshed>,
    position: WriteStorage<'a, Position>,
    monster: ReadStorage<'a, Monster>,
    player: ReadStorage<'a, Player>,
    name: ReadStorage<'a, Name>,

    runstate: Read<'a, RunState>,
    map: Read<'a, Map>,
}

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = AISystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if *data.runstate == RunState::Paused {
            return;
        }
        let player_pos: Position = *(&data.position, &data.player)
            .join()
            .map(|p| p.0)
            .next()
            .unwrap();
        for (mut viewshed, name, pos, _monster) in (
            &mut data.viewshed,
            &data.name,
            &mut data.position,
            &data.monster,
        )
            .join()
        {
            if viewshed.visible_tiles.contains(&player_pos) {
                console::log(format!("{} shouts insults!", name.name));
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
