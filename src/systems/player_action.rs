use bracket_lib::prelude::{letter_to_option, VirtualKeyCode};
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        effects::Ranged,
        in_backpack::InBackpack,
        intents::{DropIntent, MeleeIntent, PickupIntent, UseIntent},
        item::Item,
        player::Player,
        position::Position,
        viewshed::Viewshed,
    },
    resources::{
        gamelog::GameLog, input::Input, map::Map, runstate::RunState,
        shown_inventory::ShownInventory,
    },
    util::vector::{Heading, Vector},
};

#[derive(SystemData)]
pub struct PlayerActionSystemData<'a> {
    entities: Entities<'a>,

    player: ReadStorage<'a, Player>,
    combat_stats: ReadStorage<'a, CombatStats>,

    position: WriteStorage<'a, Position>,
    viewshed: WriteStorage<'a, Viewshed>,
    melee_intent: WriteStorage<'a, MeleeIntent>,
    pickup_intent: WriteStorage<'a, PickupIntent>,
    use_intent: WriteStorage<'a, UseIntent>,
    drop_intent: WriteStorage<'a, DropIntent>,
    item: ReadStorage<'a, Item>,
    backpack: ReadStorage<'a, InBackpack>,
    ranged: ReadStorage<'a, Ranged>,

    shown_inventory: Read<'a, ShownInventory>,
    input: ReadExpect<'a, Input>,

    map: WriteExpect<'a, Map>,
    gamelog: Write<'a, GameLog>,
    runstate: Write<'a, RunState>,
}

pub struct PlayerActionSystem;

enum Action {
    Move(Vector),
    PickUp,
    ShowInventory,
    ShowDropItem,
    CloseInventory,
    Use { choice: i32 },
    UseOnTarget { item: Entity, target: Position },
    Drop { choice: i32 },
    CancelTargeting,
}

impl<'a> System<'a> for PlayerActionSystem {
    type SystemData = PlayerActionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let old_runstate = *data.runstate;
        *data.runstate = match Self::resolve_action(old_runstate, &*data.input) {
            Some(Action::Move(vector)) => {
                Self::try_move_player(&mut data, vector);
                RunState::PlayerTurn
            }
            Some(Action::PickUp) => {
                Self::try_pickup(&mut data);
                RunState::PlayerTurn
            }
            Some(Action::ShowInventory) => RunState::ShowInventory,
            Some(Action::ShowDropItem) => RunState::ShowDropItem,
            Some(Action::Drop { choice }) => {
                if Self::try_drop(&mut data, choice).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::ShowDropItem
                }
            }
            Some(Action::CloseInventory) => {
                assert!(data.runstate.show_inventory());
                RunState::AwaitingInput
            }
            Some(Action::CancelTargeting) => RunState::AwaitingInput,
            Some(Action::Use { choice }) => {
                Self::try_use(&mut data, choice).unwrap_or(RunState::ShowInventory)
            }
            Some(Action::UseOnTarget { item, target }) => {
                if Self::try_use_on_target(&mut data, item, target).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::AwaitingInput
                }
            }
            None => old_runstate,
        }
    }
}

impl PlayerActionSystem {
    fn resolve_action(runstate: RunState, input: &Input) -> Option<Action> {
        match runstate {
            RunState::ShowInventory => match input.key? {
                VirtualKeyCode::Escape => Some(Action::CloseInventory),
                key => Some(Action::Use {
                    choice: letter_to_option(key),
                }),
            },
            RunState::ShowDropItem => match input.key? {
                VirtualKeyCode::Escape => Some(Action::CloseInventory),
                key => Some(Action::Drop {
                    choice: letter_to_option(key),
                }),
            },
            RunState::ShowTargeting { item, .. } => {
                if input.key == Some(VirtualKeyCode::Escape) {
                    Some(Action::CancelTargeting)
                } else if input.left_click {
                    Some(Action::UseOnTarget {
                        item,
                        target: input.mouse_pos.into(),
                    })
                } else {
                    None
                }
            }
            RunState::AwaitingInput => match input.key? {
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

                // Inventory things
                VirtualKeyCode::G => Some(Action::PickUp),
                VirtualKeyCode::I => Some(Action::ShowInventory),
                VirtualKeyCode::D => Some(Action::ShowDropItem),

                // We don't know any other keys
                _ => None,
            },
            // We don't care about keypresses during other runstates
            _ => None,
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
                        data.melee_intent
                            .insert(
                                player_entity,
                                MeleeIntent {
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
                data.pickup_intent
                    .insert(player_entity, PickupIntent { item })
                    .expect("Unable to insert want to pickup");
            }
        }
    }

    fn player_entity(data: &mut PlayerActionSystemData) -> Entity {
        (&data.player, &data.entities).join().next().unwrap().1
    }

    fn choice_to_entity_from_player_backpack(
        data: &mut PlayerActionSystemData,
        choice: i32,
    ) -> Option<Entity> {
        let player_entity = Self::player_entity(data);
        let &item = data.shown_inventory.get(choice as usize)?;
        assert_eq!(data.backpack.get(item)?.owner, player_entity);
        Some(item)
    }

    fn try_use(data: &mut PlayerActionSystemData, choice: i32) -> Option<RunState> {
        let player_entity = Self::player_entity(data);
        let item = Self::choice_to_entity_from_player_backpack(data, choice)?;
        if let Some(ranged) = data.ranged.get(item) {
            Some(RunState::ShowTargeting {
                item,
                range: ranged.range,
            })
        } else {
            data.use_intent
                .insert(player_entity, UseIntent { item, target: None })
                .expect("Failed to insert UseIntent");
            Some(RunState::PlayerTurn)
        }
    }

    fn try_use_on_target(
        data: &mut PlayerActionSystemData,
        item: Entity,
        target: Position,
    ) -> Option<()> {
        let player_entity = Self::player_entity(data);

        assert_eq!(data.backpack.get(item)?.owner, player_entity);
        data.use_intent
            .insert(
                player_entity,
                UseIntent {
                    item,
                    target: Some(target),
                },
            )
            .expect("Failed to insert UseIntent");
        Some(())
    }

    fn try_drop(data: &mut PlayerActionSystemData, choice: i32) -> Option<()> {
        let player_entity = Self::player_entity(data);
        let item = Self::choice_to_entity_from_player_backpack(data, choice)?;
        data.drop_intent
            .insert(player_entity, DropIntent { item })
            .expect("Failed to insert DropIntent");
        Some(())
    }
}
