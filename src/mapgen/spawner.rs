use crate::systems::prelude::*;

use std::collections::HashSet;

use crate::util::{random_table::RandomTable, rect_ext::RectExt};

type Spawner = fn(&mut CommandBuffer) -> Entity;

pub fn player(world: &SubWorld, position: Position, commands: &mut CommandBuffer) {
    if let Some(player_entity) = world.maybe_player_entity() {
        commands.add_component(*player_entity, position);
    } else {
        let player_entity = commands.push((
            position,
            Renderable {
                glyph: to_cp437('@'),
                color: ColorPair::new(YELLOW, BLACK),
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
            magic_mapping_scroll(commands),
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

pub fn room(
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    depth: i32,
    commands: &mut CommandBuffer,
) {
    let room_table = RandomTable::<Spawner>::new()
        .add(goblin, 10)
        .add(orc, 1 + depth)
        .add(health_potion, 7)
        .add(fireball_scroll, 2 + depth)
        .add(confusion_scroll, 2 + depth)
        .add(magic_missile_scroll, 4)
        .add(dagger, 3)
        .add(shield, 3)
        .add(long_sword, depth - 1)
        .add(tower_shield, depth - 1)
        .add(ration, 10)
        .add(magic_mapping_scroll, 2)
        .add(bear_trap, 2);
    let spawnable_count = rng.range(-2, 4 + depth);
    for position in random_positions_in_room(rng, room, spawnable_count) {
        if let Some(spawner) = room_table.roll(rng) {
            let new_entity = spawner(commands);
            commands.add_component(new_entity, position);
        }
    }
}

pub fn monster<S: ToString>(commands: &mut CommandBuffer, letter: char, name: S) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(letter),
            color: ColorPair::new(RED, BLACK),
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

pub fn orc(commands: &mut CommandBuffer) -> Entity {
    monster(commands, 'o', "Orc")
}

pub fn goblin(commands: &mut CommandBuffer) -> Entity {
    monster(commands, 'g', "Goblin")
}

pub fn random_positions_in_room(
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

pub fn health_potion(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('¡'),
            color: ColorPair::new(MAGENTA, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Health Potion".to_string()),
        Item,
        ProvidesHealing { heal_amount: 8 },
        Consumable,
        SerializeMe,
    ))
}

pub fn magic_missile_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(CYAN, BLACK),
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

pub fn fireball_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(ORANGE, BLACK),
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

pub fn confusion_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(PINK, BLACK),
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

pub fn magic_mapping_scroll(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437(')'),
            color: ColorPair::new(RGB::named(CYAN3), BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Scroll of Magic Mapping".to_string()),
        Item,
        Consumable,
        MagicMapper,
        SerializeMe,
    ))
}

pub fn dagger(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('/'),
            color: ColorPair::new(CYAN, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Dagger".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Melee),
        MeleePowerBonus::new(2),
        SerializeMe,
    ))
}

pub fn long_sword(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('/'),
            color: ColorPair::new(YELLOW, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Long Sword".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Melee),
        MeleePowerBonus::new(4),
        SerializeMe,
    ))
}

pub fn shield(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('('),
            color: ColorPair::new(CYAN, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Shield".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Shield),
        DefenseBonus::new(1),
        SerializeMe,
    ))
}

pub fn tower_shield(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('('),
            color: ColorPair::new(YELLOW, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Tower Shield".to_string()),
        Item,
        Equippable::new(EquipmentSlot::Shield),
        DefenseBonus::new(3),
        SerializeMe,
    ))
}

pub fn ration(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('%'),
            color: ColorPair::new(GREEN, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Rations".to_string()),
        Item,
        ProvidesFood,
        Consumable,
        SerializeMe,
    ))
}

pub fn bear_trap(commands: &mut CommandBuffer) -> Entity {
    commands.push((
        Renderable {
            glyph: to_cp437('^'),
            color: ColorPair::new(RED, BLACK),
            render_order: RenderOrder::Items,
        },
        Name::from("Bear Trap".to_string()),
        Hidden,
        EntryTrigger,
        InflictsDamage { damage: 6 },
        SingleActivation,
        SerializeMe,
    ))
}
