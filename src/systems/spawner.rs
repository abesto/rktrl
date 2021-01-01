use crate::systems::prelude::*;

use crossbeam_queue::SegQueue;
use std::collections::HashSet;

use crate::util::{random_table::RandomTable, rect_ext::RectExt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpawnRequest {
    Player(Position),
    Room { rect: Rect, depth: i32 },
}

#[system]
#[read_component(Entity)]
#[write_component(AreaOfEffect)]
#[write_component(BlocksTile)]
#[write_component(CombatStats)]
#[write_component(Confusion)]
#[write_component(Consumable)]
#[write_component(DefenseBonus)]
#[write_component(Equippable)]
#[write_component(HungerClock)]
#[write_component(InBackpack)]
#[write_component(InflictsDamage)]
#[write_component(Item)]
#[write_component(MeleePowerBonus)]
#[write_component(Monster)]
#[write_component(Name)]
#[write_component(Player)]
#[write_component(Position)]
#[write_component(ProvidesHealing)]
#[write_component(ProvidesFood)]
#[write_component(Ranged)]
#[write_component(Renderable)]
#[write_component(Viewshed)]
pub fn spawner(
    world: &mut SubWorld,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] spawn_requests: &mut SegQueue<SpawnRequest>,
    commands: &mut CommandBuffer,
) {
    while let Some(request) = spawn_requests.pop() {
        match request {
            SpawnRequest::Player(position) => player(world, position, commands),
            SpawnRequest::Room { rect, depth } => room(rng, &rect, depth, commands),
        }
    }
}

type Spawner = fn(&mut CommandBuffer) -> Entity;

fn player(world: &SubWorld, position: Position, commands: &mut CommandBuffer) {
    if let Some(player_entity) = world.maybe_player_entity() {
        commands.add_component(*player_entity, position);
    } else {
        let player_entity = commands.push((
            position,
            Renderable {
                glyph: to_cp437('@'),
                color: ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
                render_order: RenderOrder::Player,
            },
            Player,
            Name::from("Player".to_string()),
            Viewshed::new(8),
            BlocksTile::new(),
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
            HungerClock::default(),
        ));
        commands.add_component(player_entity, SerializeMe);

        // Wizard mode!
        let wizard_items = vec![
            health_potion(commands),
            magic_missile_scroll(commands),
            fireball_scroll(commands),
            confusion_scroll(commands),
            dagger(commands),
            shield(commands),
            ration(commands),
        ];
        for wizard_item in wizard_items {
            commands.add_component(
                wizard_item,
                InBackpack {
                    owner: player_entity,
                },
            );
        }
    }
}

fn room(rng: &mut RandomNumberGenerator, room: &Rect, depth: i32, commands: &mut CommandBuffer) {
    let room_table = RandomTable::<Spawner>::new()
        .add(goblin, 10)
        .add(orc, 1 + depth)
        .add(health_potion, 7)
        .add(fireball_scroll, 2 + depth)
        .add(confusion_scroll, 2 + depth)
        .add(confusion_scroll, 4)
        .add(dagger, 3)
        .add(long_sword, depth - 1)
        .add(shield, 3)
        .add(tower_shield, depth - 1)
        .add(ration, 10);
    let spawnable_count = rng.range(-2, 4 + depth);
    for position in random_positions_in_room(rng, room, spawnable_count) {
        if let Some(spawner) = room_table.roll(rng) {
            let new_entity = spawner(commands);
            commands.add_component(new_entity, position);
        }
    }
}

fn monster<S: ToString>(commands: &mut CommandBuffer, letter: char, name: S) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(letter),
            color: ColorPair::new(RGB::named(RED), RGB::named(BLACK)),
            render_order: RenderOrder::Monsters,
        },
        Viewshed::new(8),
        Monster,
        Name::from(name.to_string()),
        BlocksTile::new(),
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        },
        SerializeMe,
    ))
}

fn orc(commands: &mut CommandBuffer) -> Entity {
    monster(commands, 'o', "Orc")
}

fn goblin(commands: &mut CommandBuffer) -> Entity {
    monster(commands, 'g', "Goblin")
}

fn random_positions_in_room(
    rng: &mut RandomNumberGenerator,
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
            let position = rng.range(p1, p2);
            if !positions.contains(&position) {
                positions.insert(position);
                break;
            }
        }
    }

    positions
}

fn health_potion(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('¡'),
            color: ColorPair::new(RGB::named(MAGENTA), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Health Potion".to_string()),
        Item,
        ProvidesHealing { heal_amount: 8 },
        Consumable,
        SerializeMe,
    ))
}

fn magic_missile_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Magic Missile Scroll".to_string()),
        Item,
        Consumable,
        Ranged { range: 6 },
        InflictsDamage { damage: 8 },
        SerializeMe,
    ))
}

fn fireball_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(RGB::named(ORANGE), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Fireball Scroll".to_string()),
        Item,
        Consumable,
        Ranged { range: 6 },
        InflictsDamage { damage: 20 },
        AreaOfEffect { radius: 3 },
        SerializeMe,
    ))
}

fn confusion_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(RGB::named(PINK), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Confusion Scroll".to_string()),
        Item,
        Consumable,
        Ranged { range: 6 },
        Confusion { turns: 4 },
        SerializeMe,
    ))
}

fn dagger(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('/'),
            color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Dagger".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Melee),
        MeleePowerBonus::new(2),
        SerializeMe,
    ))
}

fn long_sword(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('/'),
            color: ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Long Sword".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Melee),
        MeleePowerBonus::new(4),
        SerializeMe,
    ))
}

fn shield(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('('),
            color: ColorPair::new(RGB::named(CYAN), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Shield".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Shield),
        DefenseBonus::new(1),
        SerializeMe,
    ))
}

fn tower_shield(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('('),
            color: ColorPair::new(RGB::named(YELLOW), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Tower Shield".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Shield),
        DefenseBonus::new(3),
        SerializeMe,
    ))
}

fn ration(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('%'),
            color: ColorPair::new(RGB::named(GREEN), RGB::named(BLACK)),
            render_order: RenderOrder::Items,
        },
        Name::from("Rations".to_string()),
        Item,
        ProvidesFood,
        Consumable,
        SerializeMe,
    ))
}
