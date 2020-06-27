use shipyard::*;

use crate::components::{combat_stats::CombatStats, suffer_damage::SufferDamage};

pub fn damage(mut statses: ViewMut<CombatStats>, mut suffer_damages: ViewMut<SufferDamage>) {
    for (mut stats, damage) in (&mut statses, &mut suffer_damages).iter() {
        stats.hp -= damage.amount.iter().sum::<i32>();
    }

    suffer_damages.clear();
}
