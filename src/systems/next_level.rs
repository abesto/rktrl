/// Clean up entities when moving to the next level
use legion::{component, system, systems::CommandBuffer, world::SubWorld, Entity, IntoQuery};

use crate::util::world_ext::WorldExt;
use crate::{components::*, resources::*};

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

#[system]
#[read_component(Entity)]
#[read_component(Player)]
#[read_component(InBackpack)]
#[read_component(Equipped)]
#[read_component(Player)]
#[write_component(Viewshed)]
#[write_component(CombatStats)]
pub fn next_level(
    #[resource] run_state: &RunState,
    #[resource] layout: &Layout,
    #[resource] map: &mut Map,
    #[resource] game_log: &mut GameLog,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if *run_state != RunState::NextLevel {
        return;
    }

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
    <(&mut Viewshed,)>::query()
        .filter(component::<Player>())
        .for_each_mut(world, |(viewshed,)| {
            viewshed.dirty = true;
            viewshed.revealed_tiles.clear();
        });

    // Congrats you went down
    game_log
        .entries
        .push("You descend to the next level, and take a moment to heal.".to_string());
    let old_stats = world.get_component::<CombatStats>(maybe_player_entity.unwrap());
    commands.add_component(
        maybe_player_entity.unwrap(),
        old_stats.with_hp(i32::max(old_stats.hp, old_stats.max_hp / 2)),
    );
}
