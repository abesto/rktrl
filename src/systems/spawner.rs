use std::collections::HashSet;

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use specs::shrev::*;

use crate::{
    components::{
        blocks_tile::BlocksTile,
        combat_stats::CombatStats,
        effects::{Consumable, InflictsDamage, ProvidesHealing, Ranged},
        item::Item,
        monster::Monster,
        name::Name,
        player::Player,
        position::Position,
        renderable::{RenderOrder, Renderable},
        viewshed::Viewshed,
    },
    lib::rect_ext::RectExt,
};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpawnRequest {
    Player(Position),
    Room(Rect),
}

#[derive(SystemData)]
pub struct SpawnerSystemData<'a> {
    entity: Entities<'a>,

    position: WriteStorage<'a, Position>,
    renderable: WriteStorage<'a, Renderable>,
    player: WriteStorage<'a, Player>,
    viewshed: WriteStorage<'a, Viewshed>,
    monster: WriteStorage<'a, Monster>,
    name: WriteStorage<'a, Name>,
    blocks_tile: WriteStorage<'a, BlocksTile>,
    combat_stats: WriteStorage<'a, CombatStats>,
    item: WriteStorage<'a, Item>,

    consumable: WriteStorage<'a, Consumable>,
    healing: WriteStorage<'a, ProvidesHealing>,
    ranged: WriteStorage<'a, Ranged>,
    inflicts_damage: WriteStorage<'a, InflictsDamage>,

    rng: WriteExpect<'a, RandomNumberGenerator>,
    spawn_requests: ReadExpect<'a, EventChannel<SpawnRequest>>,
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
            match request {
                SpawnRequest::Player(position) => self.player(&mut data, *position),
                SpawnRequest::Room(rect) => self.room(&mut data, rect),
            }
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
                    color: ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
                    render_order: RenderOrder::Player,
                },
                &mut data.renderable,
            )
            .with(Player, &mut data.player)
            .with(Name::from("Player".to_string()), &mut data.name)
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
                    color: ColorPair::new(RGB::named(RED), RGB::named(BLACK)),
                    render_order: RenderOrder::Monsters,
                },
                &mut data.renderable,
            )
            .with(Viewshed::new(8), &mut data.viewshed)
            .with(Monster, &mut data.monster)
            .with(Name::from(name.to_string()), &mut data.name)
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

    fn random_positions_in_room(
        &self,
        data: &mut SpawnerSystemData,
        room: &Rect,
        n: i32,
    ) -> HashSet<Position> {
        let (p1, p2) = {
            let interior = room.interior();
            (interior.p1(), interior.p2())
        };
        let mut positions: HashSet<Position> = HashSet::new();

        for _ in 0..n {
            loop {
                let position = data.rng.range(p1, p2);
                if !positions.contains(&position) {
                    positions.insert(position);
                    break;
                }
            }
        }

        positions
    }

    fn room(&self, data: &mut SpawnerSystemData, room: &Rect) {
        let num_monsters = data.rng.range(0, MAX_MONSTERS + 1);
        for position in self.random_positions_in_room(data, room, num_monsters) {
            self.random_monster(data, position);
        }

        let num_potions = data.rng.range(0, MAX_ITEMS + 1);
        for position in self.random_positions_in_room(data, room, num_potions) {
            self.random_item(data, position);
        }
    }

    fn health_potion(&self, data: &mut SpawnerSystemData, position: Position) {
        data.entity
            .build_entity()
            .with(position, &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437('¡'),
                    color: ColorPair::new(RGB::named(MAGENTA), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderable,
            )
            .with(Name::from("Health Potion".to_string()), &mut data.name)
            .with(Item, &mut data.item)
            .with(ProvidesHealing { heal_amount: 8 }, &mut data.healing)
            .with(Consumable, &mut data.consumable)
            .build();
    }

    fn magic_missile_scroll(&self, data: &mut SpawnerSystemData, position: Position) {
        data.entity
            .build_entity()
            .with(position, &mut data.position)
            .with(
                Renderable {
                    glyph: to_cp437(')'),
                    color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderable,
            )
            .with(
                Name::from("Magic Missile Scroll".to_string()),
                &mut data.name,
            )
            .with(Item, &mut data.item)
            .with(Consumable, &mut data.consumable)
            .with(Ranged { range: 6 }, &mut data.ranged)
            .with(InflictsDamage { damage: 8 }, &mut data.inflicts_damage)
            .build();
    }

    fn random_item(&self, data: &mut SpawnerSystemData, position: Position) {
        let roll: i32;
        {
            roll = data.rng.roll_dice(1, 2);
        }
        match roll {
            1 => self.health_potion(data, position),
            _ => self.magic_missile_scroll(data, position),
        }
    }
}
