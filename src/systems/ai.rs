use crate::{
    components::{
        monster::Monster, name::Name, player::Player, position::Position, viewshed::Viewshed,
    },
    resources::runstate::RunState,
};
use bracket_lib::prelude::console;
use shred_derive::SystemData;
use specs::prelude::*;
use std::collections::HashSet;

#[derive(SystemData)]
pub struct AISystemData<'a> {
    viewshed: ReadStorage<'a, Viewshed>,
    position: ReadStorage<'a, Position>,
    monster: ReadStorage<'a, Monster>,
    player: ReadStorage<'a, Player>,
    name: ReadStorage<'a, Name>,

    runstate: Read<'a, RunState>,
}

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = AISystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        if *data.runstate == RunState::Paused {
            return;
        }
        let player_pos: HashSet<&Position> =
            (&data.position, &data.player).join().map(|p| p.0).collect();
        for (viewshed, name, _monster) in (&data.viewshed, &data.name, &data.monster).join() {
            if player_pos
                .iter()
                .any(|p| viewshed.visible_tiles.contains(p))
            {
                console::log(format!("{} shouts insults!", name.name));
            }
        }
    }
}
