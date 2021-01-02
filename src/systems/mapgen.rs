use crate::systems::prelude::*;

#[system]
#[read_component(Player)]
pub fn mapgen(
    #[resource] layout: &Layout,
    #[resource] map: &mut Map,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] run_state_queue: &mut RunStateQueue,
    world: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut builder =
        crate::mapgen::random_builder(layout.map().width(), layout.map().height(), map.depth);

    builder.build_map(rng);
    builder.spawn_entities(commands, rng);

    crate::mapgen::spawner::player(world, builder.get_starting_position(), commands);
    *map = builder.get_map();

    if cfg!(feature = "visualize-mapgen") {
        run_state_queue.push_front(RunState::MapGeneration {
            snapshots: Box::new(builder.get_snapshots()),
            final_map: builder.get_map(),
            timer: 9000.0,
        });
    }
}
