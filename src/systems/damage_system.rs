use specs::prelude::*;

use crate::components::*;
use crate::resources::*;
use rktrl_macros::systemdata;

systemdata!(DamageSystemData(
    write_storage(CombatStats, SufferDamage),
    read_storage(Position),
    write_expect(Map)
));

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = DamageSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (mut stats, damage, position) in (
            &mut data.combat_statses,
            &data.suffer_damages,
            &data.positions,
        )
            .join()
        {
            stats.hp -= damage.amount.iter().sum::<i32>();
            data.map.add_bloodstain(*position);
        }

        data.suffer_damages.clear();
    }
}
