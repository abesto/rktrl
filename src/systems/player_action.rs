use bracket_lib::prelude::{letter_to_option, VirtualKeyCode};
use specs::prelude::*;

use crate::{components::*, resources::*, util::vector::*};
use rktrl_macros::systemdata;

systemdata!(PlayerActionSystemData(
    entities,
    read_storage(Player, Monster),
    write_storage(
        CombatStats,
        DropIntent,
        Equipped,
        InBackpack,
        Item,
        MeleeIntent,
        PickupIntent,
        RemoveIntent,
        Position,
        Ranged,
        UseIntent,
        Viewshed,
    ),
    read(ShownInventory, RunState),
    read_expect(Input),
    write(RunStateQueue, GameLog),
    write_expect(Map)
));

pub struct PlayerActionSystem;

enum Action {
    Move(Vector),
    SkipTurn,
    DownStairs,

    PickUp,
    ShowRemoveItem,
    ShowInventory,
    ShowDropItem,

    CloseInventory,
    Use { choice: i32 },
    UseOnTarget { item: Entity, target: Position },
    Drop { choice: i32 },
    Remove { choice: i32 },
    CancelTargeting,

    MainMenuSelect { selection: MainMenuSelection },
    NewGame,
    LoadGame,
    SaveGame,
    Quit,
}

impl<'a> System<'a> for PlayerActionSystem {
    type SystemData = PlayerActionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let old_runstate = *data.run_state;
        let new_runstate = match Self::resolve_action(old_runstate, &*data.input) {
            Some(Action::Move(vector)) => {
                Self::try_move_player(&mut data, vector);
                RunState::PlayerTurn
            }
            Some(Action::DownStairs) => {
                if Self::try_next_level(&mut data).is_some() {
                    RunState::NextLevel
                } else {
                    old_runstate
                }
            }
            Some(Action::SkipTurn) => {
                Self::skip_turn(&mut data);
                RunState::PlayerTurn
            }

            Some(Action::PickUp) => {
                Self::try_pickup(&mut data);
                RunState::PlayerTurn
            }
            Some(Action::ShowInventory) => RunState::ShowInventory,
            Some(Action::CloseInventory) => {
                assert!(data.run_state.show_inventory());
                RunState::AwaitingInput
            }

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

            Some(Action::CancelTargeting) => RunState::AwaitingInput,
            Some(Action::ShowDropItem) => RunState::ShowDropItem,
            Some(Action::Drop { choice }) => {
                if Self::try_drop(&mut data, choice).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::ShowDropItem
                }
            }

            Some(Action::ShowRemoveItem) => RunState::ShowRemoveItem,
            Some(Action::Remove { choice }) => {
                if Self::try_remove(&mut data, choice).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::ShowRemoveItem
                }
            }

            Some(Action::MainMenuSelect { selection }) => {
                old_runstate.with_main_menu_selection(selection)
            }
            Some(Action::NewGame) => RunState::PreRun,
            Some(Action::SaveGame) => RunState::SaveGame,
            Some(Action::LoadGame) => RunState::LoadGame,
            Some(Action::Quit) => {
                ::std::process::exit(0);
            }

            None => old_runstate,
        };

        if new_runstate != old_runstate {
            data.run_state_queue.push_back(new_runstate);
        }
    }
}

impl PlayerActionSystem {
    fn resolve_action(runstate: RunState, input: &Input) -> Option<Action> {
        // TODO deduplicate patterns like Down|J|Numpad2
        // (maybe only when we do a proper keymap)
        match runstate {
            state @ RunState::MainMenu { .. } => match input.key? {
                VirtualKeyCode::Down | VirtualKeyCode::J | VirtualKeyCode::Numpad2 => {
                    Some(Action::MainMenuSelect {
                        selection: state.main_menu_down(),
                    })
                }
                VirtualKeyCode::Up | VirtualKeyCode::K | VirtualKeyCode::Numpad8 => {
                    Some(Action::MainMenuSelect {
                        selection: state.main_menu_up(),
                    })
                }
                // Need .main_menu_selection() trickery due to
                // #![feature(bindings_after_at)] being unstable
                VirtualKeyCode::Return => match state.main_menu_selection() {
                    MainMenuSelection::NewGame => Some(Action::NewGame),
                    MainMenuSelection::LoadGame => Some(Action::LoadGame),
                    MainMenuSelection::Quit => Some(Action::Quit),
                },
                _ => None,
            },

            // TODO factor out "inventory choice" match arm bodies
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
            RunState::ShowRemoveItem => match input.key? {
                VirtualKeyCode::Escape => Some(Action::CloseInventory),
                key => Some(Action::Remove {
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

                // Stairs
                VirtualKeyCode::Period if input.shift => Some(Action::DownStairs),

                // Skip turn
                VirtualKeyCode::Period | VirtualKeyCode::Numpad5 => Some(Action::SkipTurn),

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
                VirtualKeyCode::R => Some(Action::ShowRemoveItem),

                // Save and exit to main menu
                VirtualKeyCode::Escape => Some(Action::SaveGame),

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
            &mut data.positions,
            &mut data.viewsheds,
            &data.players,
        )
            .join()
        {
            let new_position = data.map.clamp(*position + vector);

            if let Some(contents) = data.map.get_tile_contents(new_position) {
                for potential_target in contents.iter() {
                    if data.combat_statses.get(*potential_target).is_some() {
                        data.melee_intents
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
        let (_, player_entity, player_pos) = (&data.players, &data.entities, &data.positions)
            .join()
            .next()
            .unwrap();
        let target_item: Option<Entity> = (&data.entities, &data.items, &data.positions)
            .join()
            .find(|x| x.2 == player_pos)
            .map(|x| x.0);

        match target_item {
            None => data
                .game_log
                .entries
                .push("There is nothing here to pick up.".to_string()),
            Some(item) => {
                data.pickup_intents
                    .insert(player_entity, PickupIntent { item })
                    .expect("Unable to insert want to pickup");
            }
        }
    }

    fn player_entity(data: &mut PlayerActionSystemData) -> Entity {
        (&data.players, &data.entities).join().next().unwrap().1
    }

    fn choice_to_entity(data: &mut PlayerActionSystemData, choice: i32) -> Option<Entity> {
        let &item = data.shown_inventory.get(choice as usize)?;
        Some(item)
    }

    fn choice_to_entity_from_player_backpack(
        data: &mut PlayerActionSystemData,
        choice: i32,
    ) -> Option<Entity> {
        let maybe_item = Self::choice_to_entity(data, choice);
        if let Some(item) = maybe_item {
            let player_entity = Self::player_entity(data);
            assert_eq!(data.in_backpacks.get(item)?.owner, player_entity);
        }
        maybe_item
    }

    fn choice_to_entity_from_player_equipment(
        data: &mut PlayerActionSystemData,
        choice: i32,
    ) -> Option<Entity> {
        let maybe_item = Self::choice_to_entity(data, choice);
        if let Some(item) = maybe_item {
            let player_entity = Self::player_entity(data);
            assert_eq!(data.equippeds.get(item)?.owner, player_entity);
        }
        maybe_item
    }

    fn try_use(data: &mut PlayerActionSystemData, choice: i32) -> Option<RunState> {
        let player_entity = Self::player_entity(data);
        let item = Self::choice_to_entity_from_player_backpack(data, choice)?;
        if let Some(ranged) = data.rangeds.get(item) {
            Some(RunState::ShowTargeting {
                item,
                range: ranged.range,
            })
        } else {
            data.use_intents
                .insert(
                    player_entity,
                    UseIntent {
                        item,
                        target: UseTarget::SelfCast,
                    },
                )
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

        assert_eq!(data.in_backpacks.get(item)?.owner, player_entity);
        data.use_intents
            .insert(
                player_entity,
                UseIntent {
                    item,
                    target: UseTarget::Position(target),
                },
            )
            .expect("Failed to insert UseIntent");
        Some(())
    }

    fn try_drop(data: &mut PlayerActionSystemData, choice: i32) -> Option<()> {
        let player_entity = Self::player_entity(data);
        let item = Self::choice_to_entity_from_player_backpack(data, choice)?;
        data.drop_intents
            .insert(player_entity, DropIntent { item })
            .expect("Failed to insert DropIntent");
        Some(())
    }

    fn try_remove(data: &mut PlayerActionSystemData, choice: i32) -> Option<()> {
        let player_entity = Self::player_entity(data);
        let item = Self::choice_to_entity_from_player_equipment(data, choice)?;
        data.remove_intents
            .insert(player_entity, RemoveIntent { item })
            .expect("Failed to insert RemoveIntent");
        Some(())
    }

    fn try_next_level(data: &mut PlayerActionSystemData) -> Option<()> {
        let player_entity = Self::player_entity(data);
        let player_position = data.positions.get(player_entity)?;
        if data.map[&player_position] != TileType::DownStairs {
            data.game_log
                .entries
                .push("There is no way down from here.".to_string());
            None
        } else {
            Some(())
        }
    }

    fn skip_turn(data: &mut PlayerActionSystemData) {
        let player_entity = Self::player_entity(data);
        let viewshed = data.viewsheds.get(player_entity).unwrap();
        // Can heal if there are no visible monsters
        let can_heal: bool = !viewshed
            .visible_tiles
            .iter()
            .flat_map(|pos| data.map.get_tile_contents(*pos))
            .flatten()
            .any(|entity| data.monsters.get(*entity).is_some());
        if can_heal {
            let stats = data.combat_statses.get_mut(player_entity).unwrap();
            stats.hp = i32::min(stats.max_hp, stats.hp + 1);
        }
    }
}
