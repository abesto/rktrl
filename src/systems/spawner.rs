use std::collections::HashSet;

use bracket_lib::prelude::*;
use shred_derive::SystemData;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use specs::shrev::*;

use crate::{
    components::*,
    util::{random_table::RandomTable, rect_ext::RectExt},
};
use rktrl_macros::systemdata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpawnRequest {
    Player(Position),
    Room { rect: Rect, depth: i32 },
}

systemdata!(SpawnerSystemData(
    entities,
    write_storage(
        (serialize_me: SimpleMarker<SerializeMe>),
        AreaOfEffect,
        BlocksTile,
        CombatStats,
        Confusion,
        Consumable,
        DefenseBonus,
        Equippable,
        InBackpack,
        InflictsDamage,
        Item,
        MeleePowerBonus,
        Monster,
        Name,
        Player,
        Position,
        ProvidesHealing,
        Ranged,
        Renderable,
        Viewshed,
    ),
    write((serialize_me_alloc: SimpleMarkerAllocator<SerializeMe>)),
    write_expect((rng: RandomNumberGenerator)),
    read_expect((spawn_requests: EventChannel<SpawnRequest>))
));

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
                SpawnRequest::Room { rect, depth } => self.room(&mut data, rect, *depth),
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

type Spawner = fn(&mut SpawnerSystemData) -> Entity;

impl SpawnerSystem {
    fn player(&self, data: &mut SpawnerSystemData, position: Position) {
        if let Some((player_entity, _)) = (&data.entities, &data.players).join().next() {
            data.positions
                .insert(player_entity, position)
                .expect("Failed to set new position for player");
        } else {
            let player_entity = data
                .entities
                .build_entity()
                .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
                .with(position, &mut data.positions)
                .with(
                    Renderable {
                        glyph: to_cp437('@'),
                        color: ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
                        render_order: RenderOrder::Player,
                    },
                    &mut data.renderables,
                )
                .with(Player, &mut data.players)
                .with(Name::from("Player".to_string()), &mut data.names)
                .with(Viewshed::new(8), &mut data.viewsheds)
                .with(BlocksTile::new(), &mut data.blocks_tiles)
                .with(
                    CombatStats {
                        max_hp: 30,
                        hp: 30,
                        defense: 2,
                        power: 5,
                    },
                    &mut data.combat_statses,
                )
                .build();

            // Wizard mode!
            let wizard_items = vec![
                Self::health_potion(data),
                Self::magic_missile_scroll(data),
                Self::fireball_scroll(data),
                Self::confusion_scroll(data),
                Self::dagger(data),
                Self::shield(data),
            ];
            for wizard_item in wizard_items {
                data.in_backpacks
                    .insert(
                        wizard_item,
                        InBackpack {
                            owner: player_entity,
                        },
                    )
                    .expect("Failed to insert wizzard item");
            }
        }
    }

    fn room(&self, data: &mut SpawnerSystemData, room: &Rect, depth: i32) {
        let room_table = RandomTable::<Spawner>::new()
            .add(Self::goblin, 10)
            .add(Self::orc, 1 + depth)
            .add(Self::health_potion, 7)
            .add(Self::fireball_scroll, 2 + depth)
            .add(Self::confusion_scroll, 2 + depth)
            .add(Self::confusion_scroll, 4)
            .add(Self::dagger, 3)
            .add(Self::shield, 3);
        let spawnable_count = data.rng.range(-2, 4 + depth);
        for position in self.random_positions_in_room(data, room, spawnable_count) {
            if let Some(spawner) = room_table.roll(&mut data.rng) {
                let new_entity = spawner(data);
                data.positions
                    .insert(new_entity, position)
                    .unwrap_or_else(|_| {
                        panic!("Failed to set position for new entity {:#?}", new_entity)
                    });
            }
        }
    }

    fn monster<S: ToString>(data: &mut SpawnerSystemData, letter: char, name: S) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437(letter),
                    color: ColorPair::new(RGB::named(RED), RGB::named(BLACK)),
                    render_order: RenderOrder::Monsters,
                },
                &mut data.renderables,
            )
            .with(Viewshed::new(8), &mut data.viewsheds)
            .with(Monster, &mut data.monsters)
            .with(Name::from(name.to_string()), &mut data.names)
            .with(BlocksTile::new(), &mut data.blocks_tiles)
            .with(
                CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                },
                &mut data.combat_statses,
            )
            .build()
    }

    fn orc(data: &mut SpawnerSystemData) -> Entity {
        Self::monster(data, 'o', "Orc")
    }

    fn goblin(data: &mut SpawnerSystemData) -> Entity {
        Self::monster(data, 'g', "Goblin")
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

    fn health_potion(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437('¡'),
                    color: ColorPair::new(RGB::named(MAGENTA), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(Name::from("Health Potion".to_string()), &mut data.names)
            .with(Item, &mut data.items)
            .with(
                ProvidesHealing { heal_amount: 8 },
                &mut data.provides_healings,
            )
            .with(Consumable, &mut data.consumables)
            .build()
    }

    fn magic_missile_scroll(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437(')'),
                    color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(
                Name::from("Magic Missile Scroll".to_string()),
                &mut data.names,
            )
            .with(Item, &mut data.items)
            .with(Consumable, &mut data.consumables)
            .with(Ranged { range: 6 }, &mut data.rangeds)
            .with(InflictsDamage { damage: 8 }, &mut data.inflicts_damages)
            .build()
    }

    fn fireball_scroll(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437(')'),
                    color: ColorPair::new(RGB::named(ORANGE), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(Name::from("Fireball Scroll".to_string()), &mut data.names)
            .with(Item, &mut data.items)
            .with(Consumable, &mut data.consumables)
            .with(Ranged { range: 6 }, &mut data.rangeds)
            .with(InflictsDamage { damage: 20 }, &mut data.inflicts_damages)
            .with(AreaOfEffect { radius: 3 }, &mut data.area_of_effects)
            .build()
    }

    fn confusion_scroll(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437(')'),
                    color: ColorPair::new(RGB::named(PINK), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(Name::from("Confusion Scroll".to_string()), &mut data.names)
            .with(Item, &mut data.items)
            .with(Consumable, &mut data.consumables)
            .with(Ranged { range: 6 }, &mut data.rangeds)
            .with(Confusion { turns: 4 }, &mut data.confusions)
            .build()
    }

    fn dagger(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437('/'),
                    color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(Name::from("Dagger".to_string()), &mut data.names)
            .with(Item, &mut data.items)
            .with(Equippable::new(EquipmentSlot::Melee), &mut data.equippables)
            .with(MeleePowerBonus::new(2), &mut data.melee_power_bonuses)
            .build()
    }

    fn shield(data: &mut SpawnerSystemData) -> Entity {
        data.entities
            .build_entity()
            .marked(&mut data.serialize_me, &mut data.serialize_me_alloc)
            .with(
                Renderable {
                    glyph: to_cp437('('),
                    color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
                    render_order: RenderOrder::Items,
                },
                &mut data.renderables,
            )
            .with(Name::from("Shield".to_string()), &mut data.names)
            .with(Item, &mut data.items)
            .with(
                Equippable::new(EquipmentSlot::Shield),
                &mut data.equippables,
            )
            .with(DefenseBonus::new(1), &mut data.defense_bonuses)
            .build()
    }
}
