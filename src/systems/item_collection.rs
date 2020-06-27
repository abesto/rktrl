use shipyard::*;

use crate::{
    components::{
        in_backpack::InBackpack, intents::PickUpIntent, name::Name, player::Player,
        position::Position,
    },
    resources::gamelog::GameLog,
};

pub fn item_collection(
    entities: EntitiesView,
    mut pickups: ViewMut<PickUpIntent>,
    players: View<Player>,
    names: View<Name>,
    mut positions: ViewMut<Position>,
    mut backpacks: ViewMut<InBackpack>,
    mut gamelog: UniqueViewMut<GameLog>,
) {
    for (actor, pickup) in pickups.iter().with_id() {
        positions.delete(pickup.item);
        entities.add_component(&mut backpacks, InBackpack::new(actor.clone()), pickup.item);
        if players.contains(actor) {
            gamelog.entries.push(format!(
                "You pick up the {}.",
                names[pickup.item].to_string()
            ));
        }
    }

    pickups.clear();
}
