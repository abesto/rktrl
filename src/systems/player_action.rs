use bracket_lib::prelude::VirtualKeyCode;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, item::Item, player::Player, position::Position,
        viewshed::Viewshed, wants_to_melee::WantsToMelee, wants_to_pick_up_item::WantsToPickUpItem,
    },
    lib::vector::{Heading, Vector},
    resources::{gamelog::GameLog, input::Input, map::Map, runstate::RunState},
};

#[derive(SystemData)]
pub struct PlayerActionSystemData<'a> {
    player: ReadStorage<'a, Player>,
    combat_stats: ReadStorage<'a, CombatStats>,
    position: WriteStorage<'a, Position>,
    viewshed: WriteStorage<'a, Viewshed>,
    wants_to_melee: WriteStorage<'a, WantsToMelee>,
    wants_to_pickup: WriteStorage<'a, WantsToPickUpItem>,
    item: ReadStorage<'a, Item>,

    map: WriteExpect<'a, Map>,
    gamelog: Write<'a, GameLog>,
    input: Read<'a, Input>,
    runstate: Write<'a, RunState>,
    entities: Entities<'a>,
}

pub struct PlayerActionSystem;

enum Action {
    Move(Vector),
    PickUp,
}

impl<'a> System<'a> for PlayerActionSystem {
    type SystemData = PlayerActionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        match Self::key_to_action(data.input.key) {
            Some(Action::Move(vector)) => {
                Self::try_move_player(&mut data, vector);
                *data.runstate = RunState::PlayerTurn;
            }
            Some(Action::PickUp) => {
                Self::try_pickup(&mut data);
                *data.runstate = RunState::PlayerTurn;
            }
            None => {
                *data.runstate = RunState::AwaitingInput;
            }
        }
    }
}

impl PlayerActionSystem {
    fn key_to_action(key: Option<VirtualKeyCode>) -> Option<Action> {
        match key {
            None => None,
            Some(key) => match key {
                // Cardinal directions
                VirtualKeyCode::Up | VirtualKeyCode::K | VirtualKeyCode::Numpad8 => {
                    Some(Action::Move(Heading::North.into()))
                }

                VirtualKeyCode::Right | VirtualKeyCode::L | VirtualKeyCode::Numpad6 => {
                    Some(Action::Move(Heading::East.into()))
                }

                VirtualKeyCode::Down | VirtualKeyCode::J | VirtualKeyCode::Numpad2 => {
                    Some(Action::Move(Heading::South.into()))
                }

                VirtualKeyCode::Left | VirtualKeyCode::H | VirtualKeyCode::Numpad4 => {
                    Some(Action::Move(Heading::West.into()))
                }

                // Diagonals
                VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => {
                    Some(Action::Move(Heading::North + Heading::East))
                }
                VirtualKeyCode::Numpad7 | VirtualKeyCode::U => {
                    Some(Action::Move(Heading::North + Heading::West))
                }
                VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                    Some(Action::Move(Heading::South + Heading::East))
                }
                VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                    Some(Action::Move(Heading::South + Heading::West))
                }

                // Pick up an item
                VirtualKeyCode::G => Some(Action::PickUp),

                // We don't know any other keys
                _ => None,
            },
        }
    }

    fn try_move_player(data: &mut PlayerActionSystemData, vector: Vector) {
        for (player_entity, position, viewshed, _) in (
            &data.entities,
            &mut data.position,
            &mut data.viewshed,
            &data.player,
        )
            .join()
        {
            let new_position = data.map.clamp(*position + vector);

            if let Some(contents) = data.map.get_tile_contents(new_position) {
                for potential_target in contents.iter() {
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
            }

            if !data.map.is_blocked(new_position) {
                *position = new_position;
                viewshed.dirty = true;
            }
        }
    }

    fn try_pickup(data: &mut PlayerActionSystemData) {
        let (_, player_entity, player_pos) = (&data.player, &data.entities, &data.position)
            .join()
            .next()
            .unwrap();
        let target_item: Option<Entity> = (&data.entities, &data.item, &data.position)
            .join()
            .find(|x| x.2 == player_pos)
            .map(|x| x.0);

        match target_item {
            None => data
                .gamelog
                .entries
                .push("There is nothing here to pick up.".to_string()),
            Some(item) => {
                data.wants_to_pickup
                    .insert(
                        player_entity,
                        WantsToPickUpItem {
                            actor: player_entity,
                            item,
                        },
                    )
                    .expect("Unable to insert want to pickup");
            }
        }
    }
}
