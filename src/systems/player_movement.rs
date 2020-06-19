use bracket_lib::prelude::VirtualKeyCode;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{player::Player, position::Position, viewshed::Viewshed},
    lib::vector::{Heading, Vector},
    resources::{input::Input, map::Map, runstate::RunState},
};

#[derive(SystemData)]
pub struct PlayerMovementSystemData<'a> {
    player: ReadStorage<'a, Player>,
    position: WriteStorage<'a, Position>,
    viewshed: WriteStorage<'a, Viewshed>,

    map: Read<'a, Map>,
    input: Read<'a, Input>,
    runstate: Write<'a, RunState>,
}

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = PlayerMovementSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if let Some(heading) = Self::key_to_heading(data.input.key) {
            Self::try_move_player(&mut data, heading);
            *data.runstate = RunState::Running;
        } else {
            *data.runstate = RunState::Paused;
        }
    }
}

impl PlayerMovementSystem {
    fn key_to_heading(key: Option<VirtualKeyCode>) -> Option<Heading> {
        match key {
            None => None,
            Some(key) => match key {
                VirtualKeyCode::Up => Some(Heading::North),
                VirtualKeyCode::K => Some(Heading::North),
                VirtualKeyCode::Numpad8 => Some(Heading::North),

                VirtualKeyCode::Right => Some(Heading::East),
                VirtualKeyCode::L => Some(Heading::East),
                VirtualKeyCode::Numpad6 => Some(Heading::East),

                VirtualKeyCode::Down => Some(Heading::South),
                VirtualKeyCode::J => Some(Heading::South),
                VirtualKeyCode::Numpad2 => Some(Heading::South),

                VirtualKeyCode::Left => Some(Heading::West),
                VirtualKeyCode::H => Some(Heading::West),
                VirtualKeyCode::Numpad4 => Some(Heading::West),

                _ => None,
            },
        }
    }

    fn try_move_player(data: &mut PlayerMovementSystemData, heading: Heading) {
        for (position, viewshed, _) in (&mut data.position, &mut data.viewshed, &data.player).join()
        {
            let new_position = data.map.clamp(*position + Vector::unit(heading));
            if !data.map.is_blocked(new_position) {
                *position = new_position;
                viewshed.dirty = true;
            }
        }
    }
}
