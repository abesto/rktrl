use rktrl_macros::save_system_data;
use std::fs::File;

use serde::Serializer;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{SerializeComponents, SimpleMarker};

use crate::components::{
    blocks_tile::BlocksTile, combat_stats::CombatStats, effects::*, in_backpack::InBackpack,
    intents::*, item::Item, monster::Monster, name::Name, player::Player, position::Position,
    renderable::Renderable, serialize_me::SerializeMe, suffer_damage::SufferDamage,
    viewshed::Viewshed,
};

save_system_data!(
    BlocksTile,
    CombatStats,
    Consumable,
    ProvidesHealing,
    Ranged,
    InflictsDamage,
    AreaOfEffect,
    Confusion,
    InBackpack,
    MeleeIntent,
    PickupIntent,
    DropIntent,
    UseIntent,
    Item,
    Monster,
    Name,
    Player,
    Position,
    Renderable,
    SufferDamage,
    Viewshed,
);

pub struct SaveSystem;

impl<'a> System<'a> for SaveSystem {
    type SystemData = SaveSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let file = File::create("./savegame.ron.gz").expect("Failed to create file");
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
        let serializer =
            ron::Serializer::new(encoder, None, false).expect("Failed to create serializer");
        data.ser(serializer);
    }
}

//save_system_data!(
//            BlocksTile,
//            CombatStats,
//            Consumable,
//            ProvidesHealing,
//            Ranged,
//            InflictsDamage,
//            AreaOfEffect,
//            Confusion,
//            InBackpack,
//            MeleeIntent,
//            PickupIntent,
//            DropIntent,
//            UseIntent,
//            Item,
//            Monster,
//            Name,
//            Player,
//            Position,
//            Renderable,
//            SerializeMe,
//            SufferDamage,
//        ),
