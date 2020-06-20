use shred_derive::SystemData;
use specs::prelude::*;

use crate::{
    components::{combat_stats::CombatStats, name::Name, player::Player},
    resources::gamelog::GameLog,
};

#[derive(SystemData)]
pub struct DeathSystemData<'a> {
    entities: Entities<'a>,
    player: ReadStorage<'a, Player>,
    combat_stats: WriteStorage<'a, CombatStats>,
    name: ReadStorage<'a, Name>,
    gamelog: Write<'a, GameLog>,
}

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = DeathSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, stats, name, player) in (
            &data.entities,
            &data.combat_stats,
            &data.name,
            data.player.maybe(),
        )
            .join()
        {
            if stats.hp >= 1 {
                continue;
            }
            if player.is_none() {
                data.gamelog.entries.push(format!("{} is dead", name.name));
                data.entities.delete(entity).unwrap();
            } else {
                data.gamelog.entries.push("You are dead".to_string());
            }
        }
    }
}
