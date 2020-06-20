use bracket_lib::prelude::*;
use core::convert::TryInto;
use specs::prelude::*;

use crate::{
    resources::{gamelog::GameLog, input::Input, layout::Layout, map::Map, runstate::RunState},
    systems::{
        ai::AISystem, damage_system::DamageSystem, death::DeathSystem,
        map_indexing::MapIndexingSystem, mapgen::MapgenSystem, melee_combat::MeleeCombatSystem,
        player_movement::PlayerMovementSystem, render::RenderSystem, visibility::VisibilitySystem,
    },
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
        let runstate = *self.world.fetch::<RunState>();
        let maybe_newrunstate = match runstate {
            RunState::PreRun => {
                self.dispatcher.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
            RunState::AwaitingInput => {
                self.world.insert(Input::key(term.key));
                PlayerMovementSystem {}.run_now(&self.world);
                None
            }
            RunState::PlayerTurn => {
                self.dispatcher.dispatch(&self.world);
                Some(RunState::MonsterTurn)
            }
            RunState::MonsterTurn => {
                self.dispatcher.dispatch(&self.world);
                Some(RunState::AwaitingInput)
            }
        };

        if let Some(newrunstate) = maybe_newrunstate {
            *self.world.write_resource::<RunState>() = newrunstate;
        }

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
            .with(AISystem, "ai", &[])
            .with(VisibilitySystem, "visibility", &["ai"])
            .with(MeleeCombatSystem, "melee", &["ai"])
            .with(DamageSystem, "damage", &["melee"])
            .with(DeathSystem, "death", &["damage"])
            .with(MapIndexingSystem, "map_indexing", &["damage"])
            .build(),
        render: RenderSystem::new(),
    };
    gs.dispatcher.setup(&mut gs.world);

    // Create layout
    let layout = {
        let (width, height) = term.get_char_size();
        Layout {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            panel_height: 7,
        }
    };
    gs.world.insert(layout);

    // Generate map
    gs.world.insert({
        let map_rect = layout.map();
        Map::new(map_rect.width(), map_rect.height())
    });
    let mut init_dispatcher = DispatcherBuilder::new()
        .with(MapgenSystem::new(), "mapgen", &[])
        .build();
    init_dispatcher.setup(&mut gs.world);
    init_dispatcher.dispatch(&gs.world);

    // Welcome!
    gs.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    // And go!
    gs.world.maintain();
    main_loop(term, gs).unwrap();
}
