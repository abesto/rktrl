use bracket_lib::prelude::*;
use shipyard::*;
use std::convert::TryInto;

use crate::{
    resources::{gamelog::GameLog, input::Input, layout::Layout, map::Map, runstate::RunState},
    systems::*,
};

mod components;
mod lib;
mod resources;
mod systems;

struct State {
    world: World,
}

impl GameState for State {
    fn tick(&mut self, term: &mut BTerm) {
        let runstate = self.world.run(|runstate: UniqueView<RunState>| *runstate);
        self.world.add_unique(term.clone());
        let maybe_newrunstate = match runstate {
            RunState::PreRun => {
                self.world.run_workload("main");
                Some(RunState::AwaitingInput)
            }
            RunState::AwaitingInput | RunState::ShowInventory => {
                self.world.add_unique(Input::key(term.key));
                self.world.run(player_action);
                None
            }
            RunState::PlayerTurn => {
                self.world.run_workload("main");
                Some(RunState::MonsterTurn)
            }
            RunState::MonsterTurn => {
                self.world.run_workload("main");
                Some(RunState::AwaitingInput)
            }
        };

        if let Some(newrunstate) = maybe_newrunstate {
            self.world.add_unique(newrunstate);
        }

        self.world.run(render);
    }
}

fn main() {
    // Initialize bracket-lib
    let term = {
        let mut term = BTermBuilder::simple80x50()
            .with_title("Roguelike Tutorial")
            .build()
            .unwrap();
        term.with_post_scanlines(true);
        term
    };

    // Set up Shipyard workloads
    let world = World::new();
    world
        .add_workload("main")
        .with_system(system!(ai))
        //.with_system(system!(visibility))
        .with_system(system!(damage))
        .with_system(system!(item_collection))
        .with_system(system!(item_use))
        .with_system(system!(melee_combat))
        .with_system(system!(damage))
        .with_system(system!(death))
        .with_system(system!(map_indexing))
        .build();

    world
        .add_workload("mapgen")
        .with_system(system!(mapgen))
        //.with_system(system!(spawner))
        .build();

    // Create UI layout
    let layout = {
        let (width, height) = term.get_char_size();
        Layout {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            panel_height: 7,
        }
    };
    world.add_unique(layout);

    // Invoke RNG
    world.add_unique(RandomNumberGenerator::new());

    // Generate map
    world.add_unique({
        let map_rect = layout.map();
        Map::new(map_rect.width(), map_rect.height())
    });
    world.run_workload("mapgen");

    // Welcome!
    world.add_unique(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    // And go!
    main_loop(term, State { world }).unwrap();
}
