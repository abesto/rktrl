use shipyard::*;

use crate::{
    components::{
        combat_stats::CombatStats, intents::MeleeIntent, name::Name, suffer_damage::SufferDamage,
    },
    resources::gamelog::GameLog,
};

pub fn melee_combat(
    entities: EntitiesView,

    names: View<Name>,
    statses: View<CombatStats>,

    mut melee_intents: ViewMut<MeleeIntent>,
    mut suffer_damages: ViewMut<SufferDamage>,

    mut gamelog: UniqueViewMut<GameLog>,
) {
    for (wants_melee, name, stats) in (&melee_intents, &names, &statses).iter() {
        if stats.hp > 0 {
            let target_stats = &statses[wants_melee.target];
            if target_stats.hp > 0 {
                let target_name = &names[wants_melee.target];
                let damage = i32::max(0, stats.power - target_stats.defense);

                if damage == 0 {
                    gamelog.entries.push(format!(
                        "{} is unable to hurt {}",
                        name.to_string(),
                        target_name.to_string()
                    ));
                } else {
                    gamelog.entries.push(format!(
                        "{} hits {}, for {} hp.",
                        name.to_string(),
                        target_name.to_string(),
                        damage
                    ));
                    SufferDamage::new_damage(
                        &entities,
                        &mut suffer_damages,
                        wants_melee.target,
                        damage,
                    );
                }
            }
        }
    }

    melee_intents.clear();
}
