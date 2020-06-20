use shred_derive::SystemData;
use specs::prelude::*;

use crate::components::{combat_stats::CombatStats, suffer_damage::SufferDamage};

#[derive(SystemData)]
pub struct DamageSystemData<'a> {
    combat_stats: WriteStorage<'a, CombatStats>,
    suffer_damage: WriteStorage<'a, SufferDamage>,
}

pub struct DamageSystem;

impl DamageSystem {
    pub fn delete_the_dead(ecs: &mut World) {
        // TODO methinks this should be another system?
        let mut dead: Vec<Entity> = Vec::new();
        // Using a scope to make the borrow checker happy
        {
            let combat_stats = ecs.read_storage::<CombatStats>();
            let entities = ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp < 1 {
                    dead.push(entity);
                }
            }
        }

        for victim in dead {
            ecs.delete_entity(victim).expect("Unable to delete");
        }
    }
}

impl<'a> System<'a> for DamageSystem {
    type SystemData = DamageSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (mut stats, damage) in (&mut data.combat_stats, &data.suffer_damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        data.suffer_damage.clear();
    }
}
