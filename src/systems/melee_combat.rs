use shred_derive::SystemData;
use specs::prelude::*;

use crate::{components::*, resources::*};
use rktrl_macros::systemdata;

systemdata!(MeleeCombatSystemData(
    entities,
    write_storage(MeleeIntent, SufferDamage),
    read_storage(Name, CombatStats, Equipped, MeleePowerBonus, DefenseBonus),
    write(GameLog)
));

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (attacker, melee_intent, name, attacker_stats) in (
            &data.entities,
            &data.melee_intents,
            &data.names,
            &data.combat_statses,
        )
            .join()
        {
            if attacker_stats.hp > 0 {
                let target_stats = data.combat_statses.get(melee_intent.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = data.names.get(melee_intent.target).unwrap();

                    let power: i32 = {
                        (&data.equippeds, &data.melee_power_bonuses)
                            .join()
                            .filter(|(equipped, _)| equipped.owner == attacker)
                            .map(|(_, bonus)| bonus.power)
                            .sum::<i32>()
                            + attacker_stats.power
                    };
                    let defense: i32 = {
                        (&data.equippeds, &data.defense_bonuses)
                            .join()
                            .filter(|(equipped, _)| equipped.owner == melee_intent.target)
                            .map(|(_, bonus)| bonus.defense)
                            .sum::<i32>()
                            + target_stats.defense
                    };
                    let damage = i32::max(0, power - defense);

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
                            melee_intent.target,
                            damage,
                        );
                    }
                }
            }
        }

        data.melee_intents.clear();
    }
}
