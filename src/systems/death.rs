use rktrl_macros::systemdata;
use specs::prelude::*;

systemdata!(DeathSystemData(
    entities
    read_storage(Player, Name)
    write_storage(CombatStats)
    write(GameLog)
));

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = DeathSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, stats, name, player) in (
            &data.entities,
            &data.combat_statses,
            &data.names,
            data.players.maybe(),
        )
            .join()
        {
            if stats.hp >= 1 {
                continue;
            }
            if player.is_none() {
                data.game_log.entries.push(format!("{} is dead", name));
                data.entities.delete(entity).unwrap();
            } else {
                data.game_log.entries.push("You are dead".to_string());
            }
        }
    }
}
