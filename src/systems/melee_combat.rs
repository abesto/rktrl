use shred_derive::SystemData;
use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(MeleeCombatSystemData(
    write_storage(MeleeIntent, SufferDamage),
    read_storage(Name, CombatStats),
    write(GameLog)
));

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (wants_melee, name, stats) in
            (&data.melee_intents, &data.names, &data.combat_statses).join()
        {
            if stats.hp > 0 {
                let target_stats = data.combat_statses.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = data.names.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        data.game_log
                            .entries
                            .push(format!("{} is unable to hurt {}", name, target_name));
                    } else {
                        data.game_log
                            .entries
                            .push(format!("{} hits {}, for {} hp.", name, target_name, damage));
                        SufferDamage::new_damage(
                            &mut data.suffer_damages,
                            wants_melee.target,
                            damage,
                        );
                    }
                }
            }
        }

        data.melee_intents.clear();
    }
}
