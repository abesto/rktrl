use crate::systems::prelude::*;

#[system(for_each)]
#[read_component(Hidden)]
#[allow(clippy::too_many_arguments)]
pub fn visibility(
    #[resource] map: &Map,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] cae: &mut CauseAndEffect,
    viewshed: &mut Viewshed,
    pos: &Position,
    maybe_player: Option<&Player>,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    if viewshed.dirty {
        viewshed.visible_tiles.clear();
        viewshed.visible_tiles =
            field_of_view(Point::new(pos.x, pos.y), viewshed.range.into(), map)
                .iter()
                .map(|p| Position::from(*p))
                .filter(|p| map.contains(*p))
                .collect();
        viewshed.revealed_tiles.extend(&viewshed.visible_tiles);
        viewshed.dirty = false;
    }

    // Chance to reveal hidden things
    if maybe_player.is_some() {
        for &tile in &viewshed.visible_tiles {
            if let Some(entities) = map.get_tile_contents(tile) {
                for &entity in entities {
                    if world.has_component::<Hidden>(entity) && rng.roll_dice(1, 24) == 1 {
                        cae.add_effect(&cae.get_root(), Label::Spotted { hidden: entity });
                        commands.remove_component::<Hidden>(entity);
                    }
                }
            }
        }
    }
}
