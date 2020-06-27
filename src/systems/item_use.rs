use shipyard::*;

use crate::{
    components::{
        combat_stats::CombatStats, intents::UseIntent, name::Name, player::Player, potion::Potion,
    },
    resources::gamelog::GameLog,
};

pub fn item_use(
    players: View<Player>,
    mut use_intents: ViewMut<UseIntent>,
    potions: View<Potion>,
    names: View<Name>,
    mut statses: ViewMut<CombatStats>,
    mut gamelog: UniqueViewMut<GameLog>,
    mut all_storages: AllStoragesViewMut,
) {
    for (entity, (to_use, mut stats)) in (&use_intents, &mut statses).iter().with_id() {
        if !potions.contains(to_use.item) {
            continue;
        }
        let potion = &potions[to_use.item];
        let new_hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
        let heal_amount = new_hp - stats.hp;
        stats.hp = new_hp;
        if players.contains(entity) {
            gamelog.entries.push(format!(
                "You drink the {}, healing {} hp.",
                names[to_use.item].to_string(),
                heal_amount
            ));
        }
        all_storages.delete(to_use.item);
    }

    use_intents.clear();
}
