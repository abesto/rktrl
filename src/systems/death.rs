use shipyard::*;

use crate::{
    components::{combat_stats::CombatStats, name::Name, player::Player},
    resources::gamelog::GameLog,
};

pub fn death(
    ref mut all_storages: AllStoragesViewMut,
    ref statses: View<CombatStats>,
    ref names: View<Name>,
    ref players: View<Player>,
    ref mut gamelog: UniqueViewMut<GameLog>,
) {
    for (entity, (stats, name)) in (statses, names).iter().with_id() {
        if stats.hp >= 1 {
            continue;
        }
        if players.contains(entity) {
            gamelog
                .entries
                .push(format!("{} is dead", name.to_string()));
            all_storages.delete(entity);
        } else {
            gamelog.entries.push("You are dead".to_string());
        }
    }
}
