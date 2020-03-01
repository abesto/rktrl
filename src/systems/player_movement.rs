use crate::{
    components::{player::Player, position::Position},
    lib::vector::{Heading, Vector},
    resources::{
        input::Input,
        map::{Map, TileType},
    },
};
use bracket_lib::prelude::VirtualKeyCode;
use shred_derive::SystemData;
use specs::prelude::*;

#[derive(SystemData)]
pub struct PlayerMovementSystemData<'a> {
    player: ReadStorage<'a, Player>,
    position: WriteStorage<'a, Position>,

    map: Read<'a, Map>,
    input: Read<'a, Input>,
}

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = PlayerMovementSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if let Some(heading) = Self::key_to_heading(data.input.key) {
            Self::try_move_player(&mut data, heading);
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
        for (position, _) in (&mut data.position, &data.player).join() {
            let new_position = *position + Vector::unit(heading);
            if data.map[&new_position] == TileType::Floor {
                *position = new_position;
            }
        }
    }
}
