use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{
        combat_stats::CombatStats, intents::MeleeIntent, name::Name, suffer_damage::SufferDamage,
    },
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct MeleeCombatSystemData<'a> {
    wants_to_melee: WriteStorage<'a, MeleeIntent>,
    name: ReadStorage<'a, Name>,
    combat_stats: ReadStorage<'a, CombatStats>,
    suffer_damage: WriteStorage<'a, SufferDamage>,

    gamelog: Write<'a, GameLog>,
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
                        data.gamelog
                            .entries
                            .push(format!("{} is unable to hurt {}", name, target_name));
                    } else {
                        data.gamelog
                            .entries
                            .push(format!("{} hits {}, for {} hp.", name, target_name, damage));
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
