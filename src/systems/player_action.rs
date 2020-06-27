use bracket_lib::prelude::{letter_to_option, VirtualKeyCode};
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        in_backpack::InBackpack,
        intents::{MeleeIntent, PickupIntent, UseIntent},
        item::Item,
        player::Player,
        position::Position,
        viewshed::Viewshed,
    },
    lib::vector::{Heading, Vector},
    resources::{
        gamelog::GameLog, input::Input, map::Map, runstate::RunState,
        shown_inventory::ShownInventory,
    },
};

#[derive(SystemData)]
pub struct PlayerActionSystemData<'a> {
    player: ReadStorage<'a, Player>,
    combat_stats: ReadStorage<'a, CombatStats>,
    position: WriteStorage<'a, Position>,
    viewshed: WriteStorage<'a, Viewshed>,
    melee_intent: WriteStorage<'a, MeleeIntent>,
    pickup_intent: WriteStorage<'a, PickupIntent>,
    use_intent: WriteStorage<'a, UseIntent>,
    item: ReadStorage<'a, Item>,
    backpack: ReadStorage<'a, InBackpack>,

    map: WriteExpect<'a, Map>,
    gamelog: Write<'a, GameLog>,
    input: Read<'a, Input>,
    runstate: Write<'a, RunState>,
    entities: Entities<'a>,
    shown_inventory: Read<'a, ShownInventory>,
}

pub struct PlayerActionSystem;

enum Action {
    Move(Vector),
    PickUp,
    ShowInventory,
    CloseInventory,
    Use(i32),
}

impl<'a> System<'a> for PlayerActionSystem {
    type SystemData = PlayerActionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let old_runstate = *data.runstate;
        *data.runstate = match Self::key_to_action(old_runstate, data.input.key) {
            Some(Action::Move(vector)) => {
                Self::try_move_player(&mut data, vector);
                RunState::PlayerTurn
            }
            Some(Action::PickUp) => {
                Self::try_pickup(&mut data);
                RunState::PlayerTurn
            }
            Some(Action::ShowInventory) => RunState::ShowInventory,
            Some(Action::Use(choice)) => {
                if Self::try_use(&mut data, choice).is_some() {
                    RunState::PlayerTurn
                } else {
                    RunState::ShowInventory
                }
            }
            Some(Action::CloseInventory) => {
                assert_eq!(*data.runstate, RunState::ShowInventory);
                RunState::AwaitingInput
            }
            None => old_runstate,
        }
    }
}

impl PlayerActionSystem {
    fn key_to_action(runstate: RunState, key: Option<VirtualKeyCode>) -> Option<Action> {
        if runstate == RunState::ShowInventory {
            return match key? {
                VirtualKeyCode::Escape => Some(Action::CloseInventory),
                key => Some(Action::Use(letter_to_option(key))),
            };
        }

        match key? {
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

            // We don't know any other keys
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

    fn try_use(data: &mut PlayerActionSystemData, choice: i32) -> Option<()> {
        let &item = data.shown_inventory.get(choice as usize)?;
        let player_entity = (&data.player, &data.entities).join().next().unwrap().1;
        assert_eq!(data.backpack.get(item).unwrap().owner, player_entity);
        data.use_intent
            .insert(player_entity, UseIntent { item })
            .expect("Failed to insert WantsToUse");
        Some(())
    }
}
