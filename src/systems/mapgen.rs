use crate::systems::prelude::*;

#[system]
#[read_component(Player)]
pub fn mapgen(
    #[resource] layout: &Layout,
    #[resource] map: &mut Map,
    #[resource] rng: &mut RandomNumberGenerator,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut builder =
        crate::mapgen::random_builder(layout.map().width(), layout.map().height(), map.depth);

    builder.build_map(rng);
    builder.spawn_entities(commands, rng);

    *map = builder.get_map();
    crate::mapgen::spawner::player(world, builder.get_starting_position(), commands);
}
