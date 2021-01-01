/// Clean up entities when moving to the next level
use crate::systems::prelude::*;

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

fn equipped_to_player(
    maybe_player_entity: Option<Entity>,
    maybe_equipped: Option<&Equipped>,
) -> Option<()> {
    let player_entity = maybe_player_entity?;
    let equipped = maybe_equipped?;
    if equipped.owner == player_entity {
        Some(())
    } else {
        None
    }
}

cae_system_state!(NextLevelSystemState {
    subscribe(NextLevelIntent)
});

#[system]
#[read_component(Player)]
#[read_component(InBackpack)]
#[read_component(Equipped)]
#[read_component(Position)]
#[read_component(CombatStats)]
pub fn next_level(
    #[state] state: &NextLevelSystemState,
    #[resource] layout: &Layout,
    #[resource] map: &mut Map,
    #[resource] cae: &mut CauseAndEffect,
    #[resource] run_state_queue: &mut RunStateQueue,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    for intent in cae.get_queue(state.next_level_intent) {
        extract_nearest_ancestor!(cae, intent @ Turn => actor);
        assert!(world.is_player(actor));

        let player_position = world.get_component::<Position>(actor);
        if map[&player_position] != TileType::DownStairs {
            cae.add_effect(&intent, Label::NoStairsHere);
            continue;
        }

        run_state_queue.push_front(RunState::NextLevel);

        // Delete everything but the player and their inventory
        let maybe_player_entity = <(Entity,)>::query()
            .filter(component::<Player>())
            .iter(world)
            .next()
            .map(|x| *x.0);

        <(Entity, Option<&InBackpack>, Option<&Equipped>)>::query()
            .filter(!component::<Player>())
            .for_each(world, |(to_delete, maybe_in_backpack, maybe_equipped)| {
                if in_player_backpack(maybe_player_entity, maybe_in_backpack).is_some() {
                    return;
                }
                if equipped_to_player(maybe_player_entity, maybe_equipped).is_some() {
                    return;
                }

                commands.remove(*to_delete);
            });

        // New map who dis
        *map = {
            let map_rect = layout.map();
            Map::new(map_rect.width(), map_rect.height(), &map.depth + 1)
        };

        // You don't know this map yet
        commands.exec_mut(move |w| {
            if let Ok(viewshed) = w.entry_mut(actor).unwrap().get_component_mut::<Viewshed>() {
                viewshed.dirty = true;
                viewshed.revealed_tiles.clear();
            }
        });

        // Congrats you went down
        cae.add_effect(&intent, Label::MovedToNextLevel);
        let old_stats = world.get_component::<CombatStats>(maybe_player_entity.unwrap());
        commands.add_component(
            maybe_player_entity.unwrap(),
            old_stats.with_hp(i32::max(old_stats.hp, old_stats.max_hp / 2)),
        );
    }
}
