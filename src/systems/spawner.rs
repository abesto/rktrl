use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use specs::shrev::*;

use crate::components::{
    blocks_tile::BlocksTile, combat_stats::CombatStats, monster::Monster, name::Name,
    player::Player, position::Position, renderable::Renderable, viewshed::Viewshed,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpawnKind {
    Player,
    RandomMonster,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnRequest {
    pub position: Position,
    pub kind: SpawnKind,
}

#[derive(SystemData)]
pub struct SpawnerSystemData<'a> {
    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,
    player: WriteStorage<'a, Player>,
    viewshed: WriteStorage<'a, Viewshed>,
    monster: WriteStorage<'a, Monster>,
    name: WriteStorage<'a, Name>,
    blocks_tile: WriteStorage<'a, BlocksTile>,
    combat_stats: WriteStorage<'a, CombatStats>,

    rng: WriteExpect<'a, RandomNumberGenerator>,
    spawn_requests: ReadExpect<'a, EventChannel<SpawnRequest>>,
    entity: Entities<'a>,
}

#[derive(Default)]
pub struct SpawnerSystem {
    spawn_requests_reader: Option<ReaderId<SpawnRequest>>,
}

impl<'a> System<'a> for SpawnerSystem {
    type SystemData = SpawnerSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Clone+collect to let go of the borrow of data
        let requests: Vec<SpawnRequest> = data
            .spawn_requests
            .read(&mut self.spawn_requests_reader.as_mut().unwrap())
            .cloned()
            .collect();

        for request in requests.iter() {
            let fun = match request.kind {
                SpawnKind::Player => Self::player,
                SpawnKind::RandomMonster => Self::random_monster,
            };
            fun(&self, &mut data, request.position);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        world.insert(EventChannel::<SpawnRequest>::new());
        self.spawn_requests_reader = Some(
            world
                .fetch_mut::<EventChannel<SpawnRequest>>()
                .register_reader(),
        );
    }
}

impl SpawnerSystem {
    fn player(&self, data: &mut SpawnerSystemData, position: Position) {
        data.entity
            .build_entity()
            .with(position, &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437('@'),
                    fg: RGB::named(YELLOW),
                    bg: RGB::named(BLACK),
                },
                &mut data.renderable,
            )
            .with(Player::new(), &mut data.player)
            .with(Name::new("Player".to_string()), &mut data.name)
            .with(Viewshed::new(8), &mut data.viewshed)
            .with(BlocksTile::new(), &mut data.blocks_tile)
            .with(
                CombatStats {
                    max_hp: 30,
                    hp: 30,
                    defense: 2,
                    power: 5,
                },
                &mut data.combat_stats,
            )
            .build();
    }

    fn monster<S: ToString>(
        &self,
        data: &mut SpawnerSystemData,
        position: Position,
        letter: char,
        name: S,
    ) {
        data.entity
            .build_entity()
            .with(position, &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437(letter),
                    fg: RGB::named(RED),
                    bg: RGB::named(BLACK),
                },
                &mut data.renderable,
            )
            .with(Viewshed::new(8), &mut data.viewshed)
            .with(Monster::new(), &mut data.monster)
            .with(Name::new(name.to_string()), &mut data.name)
            .with(BlocksTile::new(), &mut data.blocks_tile)
            .with(
                CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                },
                &mut data.combat_stats,
            )
            .build();
    }

    fn orc(&self, data: &mut SpawnerSystemData, position: Position) {
        self.monster(data, position, 'o', "Orc")
    }

    fn goblin(&self, data: &mut SpawnerSystemData, position: Position) {
        self.monster(data, position, 'g', "Goblin")
    }

    fn random_monster(&self, data: &mut SpawnerSystemData, position: Position) {
        let roll: i32;
        {
            roll = data.rng.roll_dice(1, 2);
        }
        match roll {
            1 => self.orc(data, position),
            _ => self.goblin(data, position),
        }
    }
}
