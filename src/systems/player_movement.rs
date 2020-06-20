use bracket_lib::prelude::VirtualKeyCode;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, player::Player, position::Position, viewshed::Viewshed,
        wants_to_melee::WantsToMelee,
    },
    lib::vector::{Heading, Vector},
    resources::{input::Input, map::Map, runstate::RunState},
};

#[derive(SystemData)]
pub struct PlayerMovementSystemData<'a> {
    player: ReadStorage<'a, Player>,
    combat_stats: ReadStorage<'a, CombatStats>,
    position: WriteStorage<'a, Position>,
    viewshed: WriteStorage<'a, Viewshed>,
    wants_to_melee: WriteStorage<'a, WantsToMelee>,

    map: WriteExpect<'a, Map>,
    input: Read<'a, Input>,
    runstate: Write<'a, RunState>,
    entities: Entities<'a>,
}

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = PlayerMovementSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if let Some(vector) = Self::key_to_heading(data.input.key) {
            Self::try_move_player(&mut data, vector);
            *data.runstate = RunState::PlayerTurn;
        } else {
            *data.runstate = RunState::AwaitingInput;
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
        for (player_entity, position, viewshed, _) in (
            &data.entities,
            &mut data.position,
            &mut data.viewshed,
            &data.player,
        )
            .join()
        {
            let new_position = data.map.clamp(*position + vector);

            for potential_target in data.map.get_tile_contents(new_position).iter() {
                if data.combat_stats.get(*potential_target).is_some() {
                    data.wants_to_melee
                        .insert(
                            player_entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return;
                }
            }

            if !data.map.is_blocked(new_position) {
                *position = new_position;
                viewshed.dirty = true;
            }
        }
    }
}
