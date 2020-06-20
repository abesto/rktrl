use bracket_lib::prelude::console;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::components::{
    combat_stats::CombatStats, name::Name, suffer_damage::SufferDamage,
    wants_to_melee::WantsToMelee,
};

#[derive(SystemData)]
pub struct MeleeCombatSystemData<'a> {
    wants_to_melee: WriteStorage<'a, WantsToMelee>,
    name: ReadStorage<'a, Name>,
    combat_stats: ReadStorage<'a, CombatStats>,
    suffer_damage: WriteStorage<'a, SufferDamage>,
}

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (wants_melee, name, stats) in
            (&data.wants_to_melee, &data.name, &data.combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = data.combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = data.name.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        console::log(&format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        console::log(&format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(
                            &mut data.suffer_damage,
                            wants_melee.target,
                            damage,
                        );
                    }
                }
            }
        }

        data.wants_to_melee.clear();
    }
}
