use crate::systems::prelude::*;

enum Action {
    Move(Vector),
    SkipTurn,
    DownStairs,

    PickUp,
    ShowRemoveItem,
    ShowInventory,
    ShowDropItem,

    CloseInventory,
    Use {
        choice: i32,
    },
    UseOnTarget {
        item: Entity,
        target: Position,
    },
    Drop {
        choice: i32,
    },
    Remove {
        choice: i32,
    },
    CancelTargeting,

    MainMenuSelect {
        selection: MainMenuSelection,
    },
    NewGame,
    LoadGame,
    SaveGame,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
    Restart,
}

#[system]
#[read_component(Position)]
#[read_component(Item)]
#[read_component(InBackpack)]
#[read_component(Equipped)]
#[read_component(Ranged)]
#[read_component(Monster)]
#[read_component(HungerClock)]
#[read_component(Entity)]
#[write_component(CombatStats)]
#[write_component(Player)]
#[write_component(Viewshed)]
#[allow(clippy::too_many_arguments)]
pub fn player_action(
    #[resource] run_state: &RunState,
    #[resource] input: &Input,
    #[resource] map: &Map,
    #[resource] shown_inventory: &ShownInventory,
    #[resource] run_state_queue: &mut RunStateQueue,
    #[resource] cae: &mut CauseAndEffect,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if let Some(cause) = if let Some(player_entity) = world.maybe_player_entity() {
        cae.find_first_link(|link| match link.label {
            Label::Turn { actor: entity } => entity == *player_entity,
            _ => false,
        })
    } else {
        cae.find_first_link(|link| link.label == Label::Root)
    } {
        let input_link = cae.add_effect(&cause, Label::Input { input: *input });

        let old_runstate = *run_state;
        let new_runstate = match resolve_action(old_runstate, input) {
            Some(Action::Move(vector)) => {
                try_move_player(world, cae, &input_link, map, vector);
                RunState::PlayerTurn
            }
            Some(Action::DownStairs) => {
                cae.add_effect(&input_link, Label::NextLevelIntent);
                RunState::PlayerTurn
            }
            Some(Action::SkipTurn) => {
                cae.add_effect(&input_link, Label::SkipBecauseInput);
                skip_turn(world, commands, map);
                RunState::PlayerTurn
            }

            Some(Action::PickUp) => {
                cae.add_effect(&input_link, Label::PickupIntent);
                RunState::PlayerTurn
            }
            Some(Action::ShowInventory) => RunState::ShowInventory,
            Some(Action::CloseInventory) => {
                assert!(run_state.show_inventory());
                RunState::AwaitingInput
            }

            Some(Action::Use { choice }) => {
                try_use(world, cae, &input_link, shown_inventory, choice)
                    .unwrap_or(RunState::ShowInventory)
            }
            Some(Action::UseOnTarget { item, target }) => {
                if try_use_on_target(world, cae, &input_link, item, target).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::AwaitingInput
                }
            }

            Some(Action::CancelTargeting) => RunState::AwaitingInput,
            Some(Action::ShowDropItem) => RunState::ShowDropItem,
            Some(Action::Drop { choice }) => {
                if try_drop(world, cae, &input_link, shown_inventory, choice).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::ShowDropItem
                }
            }

            Some(Action::ShowRemoveItem) => RunState::ShowRemoveItem,
            Some(Action::Remove { choice }) => {
                if try_remove(world, cae, &input_link, shown_inventory, choice).is_some() {
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
            #[cfg(not(target_arch = "wasm32"))]
            Some(Action::Quit) => {
                ::std::process::exit(0);
            }

            Some(Action::Restart) => RunState::default(),

            None => old_runstate,
        };

        if new_runstate != old_runstate {
            run_state_queue.push_back(new_runstate);
        }
    }

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
                    #[cfg(not(target_arch = "wasm32"))]
                    MainMenuSelection::Quit => Some(Action::Quit),
                },
                _ => None,
            },

            RunState::GameOver => {
                if input.key.is_some() {
                    Some(Action::Restart)
                } else {
                    None
                }
            }

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
            // We don't care about key presses during other runstates
            _ => None,
        }
    }
}

fn try_move_player(
    world: &mut SubWorld,
    cae: &mut CauseAndEffect,
    cause: &Link,
    map: &Map,
    vector: Vector,
) {
    let position = world.player_component::<Position>();
    let new_position = map.clamp(position + vector);

    if let Some(contents) = map.get_tile_contents(new_position) {
        for potential_target in contents.iter() {
            if world.has_component::<CombatStats>(*potential_target) {
                cae.add_effect(
                    &cause,
                    Label::MeleeIntent {
                        target_position: new_position,
                    },
                );
                return;
            }
        }
    }

    cae.add_effect(
        &cause,
        Label::MoveIntent {
            target_position: new_position,
        },
    );
}

fn choice_to_entity(shown_inventory: &ShownInventory, choice: i32) -> Option<Entity> {
    let &item = shown_inventory.get(choice as usize)?;
    Some(item)
}

fn choice_to_entity_from_player_backpack(
    world: &SubWorld,
    shown_inventory: &ShownInventory,
    choice: i32,
) -> Option<Entity> {
    let maybe_item = choice_to_entity(shown_inventory, choice);
    if let Some(item) = maybe_item {
        let player_entity = *world.player_entity();
        assert!(world.has_component::<InBackpack>(item));
        assert_eq!(world.get_component::<InBackpack>(item).owner, player_entity);
    }
    maybe_item
}

fn choice_to_entity_from_player_equipment(
    world: &SubWorld,
    shown_inventory: &ShownInventory,
    choice: i32,
) -> Option<Entity> {
    let maybe_item = choice_to_entity(shown_inventory, choice);
    if let Some(item) = maybe_item {
        let player_entity = *world.player_entity();
        assert!(world.has_component::<Equipped>(item));
        assert_eq!(world.get_component::<Equipped>(item).owner, player_entity);
    }
    maybe_item
}

fn try_use(
    world: &mut SubWorld,
    cae: &mut CauseAndEffect,
    cause: &Link,
    shown_inventory: &ShownInventory,
    choice: i32,
) -> Option<RunState> {
    let item = choice_to_entity_from_player_backpack(world, shown_inventory, choice)?;
    if let Ok(ranged) = world.entry_ref(item).unwrap().get_component::<Ranged>() {
        Some(RunState::ShowTargeting {
            item,
            range: ranged.range,
        })
    } else {
        cae.add_effect(
            cause,
            Label::UseIntent {
                item,
                target: UseTarget::SelfCast,
            },
        );
        Some(RunState::PlayerTurn)
    }
}

fn try_use_on_target(
    world: &mut SubWorld,
    cae: &mut CauseAndEffect,
    cause: &Link,
    item: Entity,
    target: Position,
) -> Option<()> {
    let player_entity = *world.player_entity();

    assert_eq!(
        world
            .entry_ref(item)
            .unwrap()
            .get_component::<InBackpack>()
            .ok()?
            .owner,
        player_entity
    );
    cae.add_effect(
        cause,
        Label::UseIntent {
            item,
            target: UseTarget::Position(target),
        },
    );
    Some(())
}

fn try_drop(
    world: &mut SubWorld,
    cae: &mut CauseAndEffect,
    cause: &Link,
    shown_inventory: &ShownInventory,
    choice: i32,
) -> Option<()> {
    let item = choice_to_entity_from_player_backpack(world, shown_inventory, choice)?;
    cae.add_effect(cause, Label::DropIntent { item: item });
    Some(())
}

fn try_remove(
    world: &mut SubWorld,
    cae: &mut CauseAndEffect,
    cause: &Link,
    shown_inventory: &ShownInventory,
    choice: i32,
) -> Option<()> {
    let item = choice_to_entity_from_player_equipment(world, shown_inventory, choice)?;
    cae.add_effect(cause, Label::RemoveIntent { item });
    Some(())
}

fn skip_turn(world: &mut SubWorld, commands: &mut CommandBuffer, map: &Map) {
    let player_entity = *world.player_entity();
    let player_entry = world.entry_ref(player_entity).unwrap();

    let monsters_visible: bool = player_entry
        .get_component::<Viewshed>()
        .unwrap()
        .visible_tiles
        .iter()
        .flat_map(|pos| map.get_tile_contents(*pos))
        .flatten()
        .any(|entity| world.has_component::<Monster>(*entity));

    let hungry: bool = matches!(
        player_entry.get_component::<HungerClock>().unwrap().state,
        HungerState::Hungry | HungerState::Starving
    );

    let can_heal = !monsters_visible && !hungry;

    if can_heal {
        commands.exec_mut(move |w| {
            let mut entry = w.entry_mut(player_entity).unwrap();
            let stats: &mut CombatStats = entry.get_component_mut().unwrap();
            stats.hp = i32::min(stats.max_hp, stats.hp + 1);
        });
    }
}
