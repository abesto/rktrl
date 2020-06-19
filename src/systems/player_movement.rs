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
        if let Some(vector) = Self::key_to_heading(data.input.key) {
            Self::try_move_player(&mut data, vector);
            *data.runstate = RunState::Running;
        } else {
            *data.runstate = RunState::Paused;
        }
    }
}

impl PlayerMovementSystem {
    fn key_to_heading(key: Option<VirtualKeyCode>) -> Option<Vector> {
        match key {
            None => None,
            Some(key) => match key {
                // Cardinal directions
                VirtualKeyCode::Up | VirtualKeyCode::K | VirtualKeyCode::Numpad8 => {
                    Some(Heading::North.into())
                }

                VirtualKeyCode::Right | VirtualKeyCode::L | VirtualKeyCode::Numpad6 => {
                    Some(Heading::East.into())
                }

                VirtualKeyCode::Down | VirtualKeyCode::J | VirtualKeyCode::Numpad2 => {
                    Some(Heading::South.into())
                }

                VirtualKeyCode::Left | VirtualKeyCode::H | VirtualKeyCode::Numpad4 => {
                    Some(Heading::West.into())
                }

                // Diagonals
                VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => Some(Heading::North + Heading::East),
                VirtualKeyCode::Numpad7 | VirtualKeyCode::U => Some(Heading::North + Heading::West),
                VirtualKeyCode::Numpad3 | VirtualKeyCode::N => Some(Heading::South + Heading::East),
                VirtualKeyCode::Numpad1 | VirtualKeyCode::B => Some(Heading::South + Heading::West),

                _ => None,
            },
        }
    }

    fn try_move_player(data: &mut PlayerMovementSystemData, vector: Vector) {
        for (position, viewshed, _) in (&mut data.position, &mut data.viewshed, &data.player).join()
        {
            let new_position = data.map.clamp(*position + vector);
            if !data.map.is_blocked(new_position) {
                *position = new_position;
                viewshed.dirty = true;
            }
        }
    }
}
