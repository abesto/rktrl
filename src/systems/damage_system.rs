use shred_derive::SystemData;
use specs::prelude::*;

use crate::components::{combat_stats::CombatStats, suffer_damage::SufferDamage};

#[derive(SystemData)]
pub struct DamageSystemData<'a> {
    combat_stats: WriteStorage<'a, CombatStats>,
    suffer_damage: WriteStorage<'a, SufferDamage>,
}

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = DamageSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (mut stats, damage) in (&mut data.combat_stats, &data.suffer_damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        data.suffer_damage.clear();
    }
}
