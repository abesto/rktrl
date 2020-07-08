use specs::prelude::*;

use crate::components::*;
use rktrl_macros::systemdata;

systemdata!(DamageSystemData(write_storage(CombatStats, SufferDamage)));

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = DamageSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (mut stats, damage) in (&mut data.combat_statses, &data.suffer_damages).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        data.suffer_damages.clear();
    }
}
