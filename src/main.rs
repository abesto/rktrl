use bracket_lib::prelude::*;
use specs::prelude::*;

mod components;
mod resources;
mod systems;

use crate::systems::{mapgen::MapgenSystem, render::RenderSystem};

struct State {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    render: RenderSystem,
}

impl GameState for State {
    fn tick(&mut self, term: &mut BTerm) {
        self.dispatcher.dispatch(&mut self.world);
        // RenderSystem needs special treatment (see RenderSystem::run)
        self.render.run_now_with_term(&mut self.world, term);
        self.world.maintain();
    }
}

fn main() {
    // Initialize bracket-lib
    let term = BTermBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();

    // Initialize specs
    let mut gs = State {
        world: World::new(),
        dispatcher: DispatcherBuilder::new().build(),
        render: RenderSystem::new(),
    };
    gs.dispatcher.setup(&mut gs.world);

    // One-off startup
    let mut init_dispatcher = DispatcherBuilder::new()
        .with(MapgenSystem::new(), "mapgen", &[])
        .build();
    init_dispatcher.setup(&mut gs.world);
    init_dispatcher.dispatch(&mut gs.world);
    gs.world.maintain();

    // And go!
    main_loop(term, gs);
}
