use bracket_lib::prelude::*;
use legion::{Entity, EntityStore, IntoQuery, system, systems::CommandBuffer, world::SubWorld};

use crate::{
    components::*, resources::*, systems::particle::ParticleRequests, util::world_ext::WorldExt,
};

// TODO convert to for_each system after https://github.com/amethyst/legion/issues/199 is fixed
#[system]
// for_each components
#[read_component(Entity)]
#[read_component(MeleeIntent)]
#[read_component(Name)]
#[read_component(CombatStats)]
#[read_component(HungerClock)]
// eof for_each components
#[read_component(Equipped)]
#[read_component(MeleePowerBonus)]
#[read_component(DefenseBonus)]
#[read_component(Position)]
#[allow(clippy::too_many_arguments)]
pub fn melee_combat(
    #[resource] game_log: &mut GameLog,
    #[resource] particle_requests: &mut ParticleRequests,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    <(
        Entity,
        &MeleeIntent,
        &Name,
        &CombatStats,
        Option<&HungerClock>,
    )>::query()
        .for_each(
            world,
            |(attacker, melee_intent, name, attacker_stats, maybe_attacker_hunger_clock)| {
                commands.remove_component::<MeleeIntent>(*attacker);
                let target_stats = world.get_component::<CombatStats>(melee_intent.target);
                let target_name = world.get_component::<Name>(melee_intent.target);

                if attacker_stats.hp <= 0 || target_stats.hp <= 0 {
                    return;
                }

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
                    <(&Equipped, &MeleePowerBonus)>::query()
                        .iter(world)
                        .filter(|(equipped, _)| equipped.owner == *attacker)
                        .map(|(_, bonus)| bonus.power)
                        .sum::<i32>()
                        + attacker_stats.power
                        + hunger_attack_power_bonus
                };
                let defense: i32 = {
                    <(&Equipped, &DefenseBonus)>::query()
                        .iter(world)
                        .filter(|(equipped, _)| equipped.owner == melee_intent.target)
                        .map(|(_, bonus)| bonus.defense)
                        .sum::<i32>()
                        + target_stats.defense
                };
                let damage = i32::max(0, power - defense);

                if damage == 0 {
                    game_log
                        .entries
                        .push(format!("{} is unable to hurt {}", name, target_name));
                } else {
                    game_log
                        .entries
                        .push(format!("{} hits {}, for {} hp.", name, target_name, damage));
                    SufferDamage::new_damage(commands, melee_intent.target, damage);
                }

                if let Ok(position) = world
                    .entry_ref(melee_intent.target)
                    .unwrap()
                    .get_component::<Position>()
                {
                    particle_requests.request(
                        position.x,
                        position.y,
                        RGB::named(ORANGE),
                        RGB::named(BLACK),
                        to_cp437('‼'),
                        200.0,
                    )
                }
            },
        );
}
