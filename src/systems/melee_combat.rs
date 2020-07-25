use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;

use crate::{components::*, resources::*, systems::particle::ParticleRequests};
use rktrl_macros::systemdata;

systemdata!(MeleeCombatSystemData(
    entities,
    write_storage(MeleeIntent, SufferDamage),
    read_storage(
        Name,
        CombatStats,
        Equipped,
        HungerClock,
        MeleePowerBonus,
        DefenseBonus,
        Position
    ),
    write(GameLog, ParticleRequests)
));

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = MeleeCombatSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (attacker, melee_intent, name, attacker_stats, maybe_attacker_hunger_clock) in (
            &data.entities,
            &data.melee_intents,
            &data.names,
            &data.combat_statses,
            data.hunger_clocks.maybe(),
        )
            .join()
        {
            if attacker_stats.hp > 0 {
                let target_stats = data.combat_statses.get(melee_intent.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = data.names.get(melee_intent.target).unwrap();

                    let hunger_attack_power_bonus = maybe_attacker_hunger_clock
                        .map(|clock| {
                            if clock.state == HungerState::WellFed {
                                1
                            } else {
                                0
                            }
                        })
                        .unwrap_or(0);

                    let power: i32 = {
                        (&data.equippeds, &data.melee_power_bonuses)
                            .join()
                            .filter(|(equipped, _)| equipped.owner == attacker)
                            .map(|(_, bonus)| bonus.power)
                            .sum::<i32>()
                            + attacker_stats.power
                            + hunger_attack_power_bonus
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

                    if let Some(position) = data.positions.get(melee_intent.target) {
                        data.particle_requests.request(
                            position.x,
                            position.y,
                            RGB::named(ORANGE),
                            RGB::named(BLACK),
                            to_cp437('‼'),
                            200.0,
                        )
                    }
                }
            }
        }

        data.melee_intents.clear();
    }
}
