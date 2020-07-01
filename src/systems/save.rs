use std::fs::File;

use rktrl_macros::save_system_data;
use serde::{Deserialize, Serialize, Serializer};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    ConvertSaveload, MarkedBuilder, Marker, SerializeComponents, SimpleMarker,
    SimpleMarkerAllocator,
};
use specs_derive::{Component, ConvertSaveload};

use crate::{
    components::{
        blocks_tile::BlocksTile, combat_stats::CombatStats, effects::*, in_backpack::InBackpack,
        intents::*, item::Item, monster::Monster, name::Name, player::Player, position::Position,
        renderable::Renderable, serialize_me::SerializeMe, suffer_damage::SufferDamage,
        viewshed::Viewshed,
    },
    resources::{gamelog::GameLog, map::Map},
};

#[derive(Clone, Component, ConvertSaveload)]
pub struct SerializationHelper {
    map: Map,
    gamelog: GameLog,
}

save_system_data!(
    components(
        SerializationHelper,
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
    )
    resources(Map, GameLog)
);

pub struct SaveSystem;

impl SaveSystem {
    /// Unfortunately SerializeComponents::serialize requires a ReadStorage over the
    /// serialization marker. We need to write this marker when prepping an entity to
    /// serialize resources to, so we have a conflict: we can't have both a ReadStorage
    /// and a WriteStorage. This workaround breaks out the preparation step.
    pub fn prepare(world: &mut World) {
        let map = (*world.fetch::<Map>()).clone();
        let gamelog = (*world.fetch::<GameLog>()).clone();
        world
            .create_entity()
            .with(SerializationHelper { map, gamelog })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}

impl<'a> System<'a> for SaveSystem {
    type SystemData = SaveSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let serialization_helper = &(data.components.0).0;
        assert_eq!((serialization_helper,).join().count(), 1);

        let file = File::create("./savegame.ron.gz").expect("Failed to create file");
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
        let serializer =
            ron::Serializer::new(encoder, None, false).expect("Failed to create serializer");
        data.ser(serializer);

        // Clean up serialization helper entities
        for (entity, _) in (&data.entities, serialization_helper).join() {
            data.entities
                .delete(entity)
                .expect("Failed to clean up serialization helper");
        }
        assert_eq!((serialization_helper,).join().count(), 0);
    }
}
