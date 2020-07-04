/// Clean up entities when moving to the next level
use specs::prelude::*;

use rktrl_macros::systemdata;

systemdata!(NextLevelSystemData(
    entities
    read(RunState)
    read_storage(Player, InBackpack)
    write_storage(Viewshed, CombatStats)
    read_expect(Layout)
    write_expect(Map, GameLog)
));

pub struct NextLevelSystem;

fn in_player_backpack(
    maybe_player_entity: Option<Entity>,
    maybe_in_backpack: Option<&InBackpack>,
) -> Option<()> {
    let player_entity = maybe_player_entity?;
    let in_backpack = maybe_in_backpack?;
    if in_backpack.owner == player_entity {
        Some(())
    } else {
        None
    }
}

impl<'a> System<'a> for NextLevelSystem {
    type SystemData = NextLevelSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if *data.run_state != RunState::NextLevel {
            return;
        }

        // Delete everything but the player and their inventory
        let maybe_player_entity = (&data.entities, &data.players).join().next().map(|x| x.0);
        for (to_delete, maybe_in_backpack, _) in
            (&data.entities, data.in_backpacks.maybe(), !&data.players).join()
        {
            if in_player_backpack(maybe_player_entity, maybe_in_backpack).is_some() {
                continue;
            }

            data.entities
                .delete(to_delete)
                .expect("Failed to delete entity in cleanup");
        }

        // New map who dis
        *data.map = {
            let map_rect = data.layout.map();
            Map::new(map_rect.width(), map_rect.height(), data.map.depth + 1)
        };

        // You don't know this map yet
        for (mut viewshed, _) in (&mut data.viewsheds, &data.players).join() {
            viewshed.dirty = true;
            viewshed.revealed_tiles.clear();
        }

        // Congrats you went down
        data.game_log
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        if let Some(player_stats) =
            maybe_player_entity.and_then(|player_entity| data.combat_statses.get_mut(player_entity))
        {
            player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);
        }
    }
}
