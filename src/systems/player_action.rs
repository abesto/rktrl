use bracket_lib::prelude::{letter_to_option, VirtualKeyCode};
use shipyard::*;

use crate::{
    components::{
        combat_stats::CombatStats,
        in_backpack::InBackpack,
        intents::{MeleeIntent, PickUpIntent, UseIntent},
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

enum Action {
    Move(Vector),
    PickUp,
    ShowInventory,
    CloseInventory,
    Use(i32),
}

pub fn player_action(
    mut all_storages: AllStoragesViewMut,
    entities: EntitiesView,

    players: View<Player>,
    statses: View<CombatStats>,
    items: View<Item>,
    backpacks: View<InBackpack>,

    mut positions: ViewMut<Position>,
    mut viewsheds: ViewMut<Viewshed>,
    mut melee_intents: ViewMut<MeleeIntent>,
    mut pickup_intents: ViewMut<PickUpIntent>,
    mut use_intents: ViewMut<UseIntent>,

    input: UniqueView<Input>,
    shown_inventory: UniqueView<ShownInventory>,
    map: UniqueView<Map>,
    mut runstate: UniqueViewMut<RunState>,
    mut gamelog: UniqueViewMut<GameLog>,
) {
    let try_pickup = || {
        let (player_entity, (_, player_pos)) =
            (&players, &positions).iter().with_id().next().unwrap();
        let target_item: Option<EntityId> = (&items, &positions)
            .iter()
            .with_id()
            .find(|(id, x)| x.1 == player_pos)
            .map(|(id, _)| id);

        match target_item {
            None => gamelog
                .entries
                .push("There is nothing here to pick up.".to_string()),
            Some(item) => {
                entities.add_component(&mut pickup_intents, PickUpIntent { item }, player_entity);
            }
        }
    };

    let try_use = |choice: i32| -> Option<()> {
        let &item = shown_inventory.get(choice as usize)?;
        let player_entity = players.iter().with_id().next().unwrap().0;
        assert_eq!(backpacks[item].owner, player_entity);
        entities.add_component(&mut use_intents, UseIntent { item }, player_entity);
        Some(())
    };

    let old_runstate = (*runstate).clone();
    *runstate = match key_to_action(old_runstate, input.key) {
        Some(Action::Move(vector)) => {
            try_move_player(vector);
            RunState::PlayerTurn
        }
        Some(Action::PickUp) => {
            try_pickup();
            RunState::PlayerTurn
        }
        Some(Action::ShowInventory) => RunState::ShowInventory,
        Some(Action::Use(choice)) => {
            if try_use(choice).is_some() {
                RunState::PlayerTurn
            } else {
                RunState::ShowInventory
            }
        }
        Some(Action::CloseInventory) => {
            assert_eq!(old_runstate, RunState::ShowInventory);
            RunState::AwaitingInput
        }
        None => old_runstate,
    }
}

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


fn try_move_player() {
for (player_entity, (position, mut viewshed, _)) in
(&mut positions, &mut viewsheds, &players).iter().with_id()
{
let new_position = map.clamp(*position + vector);

if let Some(contents) = map.get_tile_contents(new_position) {
for potential_target in contents.iter() {
if statses.contains(*potential_target) {
entities.add_component(
&mut melee_intents,
MeleeIntent {
target: *potential_target,
},
player_entity,
);
return;
}
}
}

if !map.is_blocked(new_position) {
*position = new_position;
viewshed.dirty = true;
}
}
}
};

