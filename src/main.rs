use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::resources::input::Input;
use crate::systems::{
    ai::AISystem, map_indexing::MapIndexingSystem, mapgen::MapgenSystem,
    player_movement::PlayerMovementSystem, render::RenderSystem, visibility::VisibilitySystem,
};

mod components;
mod lib;
mod resources;
mod systems;

struct State {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    render: RenderSystem,
}

impl GameState for State {
    fn tick(&mut self, term: &mut BTerm) {
        self.world.insert(Input::key(term.key));
        self.dispatcher.dispatch(&self.world);
        // RenderSystem needs special treatment (see RenderSystem::run)
        self.render.run_now_with_term(&mut self.world, term);
        self.world.maintain();
    }
}

fn main() {
    // Initialize bracket-lib
    let term = BTermBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()
        .unwrap();

    // Initialize specs
    let mut gs = State {
        world: World::new(),
        dispatcher: DispatcherBuilder::new()
            .with(PlayerMovementSystem, "player_movement", &[])
            .with(AISystem, "ai", &["player_movement"])
            .with(VisibilitySystem, "visibility", &["player_movement", "ai"])
            .with(
                MapIndexingSystem,
                "map_indexing",
                &["ai", "player_movement"],
            )
            .build(),
        render: RenderSystem::new(),
    };
    gs.dispatcher.setup(&mut gs.world);

    // One-off startup
    let mut init_dispatcher = DispatcherBuilder::new()
        .with(MapgenSystem::new(), "mapgen", &[])
        .build();
    init_dispatcher.setup(&mut gs.world);
    init_dispatcher.dispatch(&gs.world);
    gs.world.maintain();

    // And go!
    main_loop(term, gs).unwrap();
}
