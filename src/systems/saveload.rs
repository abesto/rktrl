#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(target_arch = "wasm32")]
use std::io::{Cursor, Error, ErrorKind, Read, Result as IOResult, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};

use rktrl_macros::saveload_system_data;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use specs::error::NoError;
use specs::prelude::{Write as SpecsWrite, *};
use specs::saveload::{
    ConvertSaveload, DeserializeComponents, MarkedBuilder, Marker, SerializeComponents,
    SimpleMarker, SimpleMarkerAllocator,
};
use specs_derive::{Component, ConvertSaveload};

use crate::{components::*, resources::*};

#[cfg(not(target_arch = "wasm32"))]
const SAVEGAME: &str = "./savegame.ron.gz";
#[cfg(target_arch = "wasm32")]
const SAVEGAME: &str = "savegame";

#[derive(Clone, Component, ConvertSaveload)]
pub struct SerializationHelper {
    map: Map,
    game_log: GameLog,
}

saveload_system_data!(
    components(
        SerializationHelper,
        // -- Sort below this -- //
        AreaOfEffect,
        BlocksTile,
        CombatStats,
        Confusion,
        Consumable,
        DefenseBonus,
        Equippable,
        Equipped,
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
        SufferDamage,
        Viewshed,
    )
    resources(Map, GameLog)
);

#[cfg(target_arch = "wasm32")]
pub struct LocalStorageWriter {
    buffer: Vec<u8>,
    storage: stdweb::web::Storage,
}

#[cfg(target_arch = "wasm32")]
impl LocalStorageWriter {
    fn new() -> LocalStorageWriter {
        LocalStorageWriter {
            buffer: vec![],
            storage: stdweb::web::window().local_storage(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::io::Write for LocalStorageWriter {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.buffer.append(&mut buf.to_vec());
        Ok(buf.len())
    }

    fn flush(&mut self) -> IOResult<()> {
        let encoded = base64::encode(&self.buffer);
        print!("{}", encoded);
        self.buffer.clear();
        self.storage
            .insert(SAVEGAME, &encoded)
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to write into local storage"))
    }
}

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
            .with(SerializationHelper {
                map,
                game_log: gamelog,
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn writer() -> File {
        File::create(SAVEGAME).expect("Failed to create file")
    }

    #[cfg(target_arch = "wasm32")]
    fn writer() -> LocalStorageWriter {
        LocalStorageWriter::new()
    }
}

impl<'a> System<'a> for SaveSystem {
    type SystemData = SaveSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let serialization_helper = &(data.components.0).0;
        assert_eq!((serialization_helper,).join().count(), 1);

        let mut writer = Self::writer();
        let encoder = flate2::write::GzEncoder::new(&mut writer, flate2::Compression::fast());
        let serializer =
            ron::Serializer::new(encoder, None, false).expect("Failed to create serializer");
        data.ser(serializer);
        writer.flush().expect("Failed to flush savegame");

        // Clean up serialization helper entities
        for (entity, _) in (&data.entities, serialization_helper).join() {
            data.entities
                .delete(entity)
                .expect("Failed to clean up serialization helper");
        }
    }
}

pub struct LoadSystem;

impl LoadSystem {
    #[cfg(not(target_arch = "wasm32"))]
    fn reader() -> File {
        File::open(SAVEGAME).expect("Failed to open file")
    }

    #[cfg(target_arch = "wasm32")]
    fn reader() -> Cursor<Vec<u8>> {
        let storage = stdweb::web::window().local_storage();
        let encoded = storage
            .get(SAVEGAME)
            .expect("Failed to read from local storage")
            .into_bytes();
        Cursor::new(base64::decode(encoded).expect("Failed to base64 decode savegame"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn savegame_exists() -> bool {
        std::path::Path::new(SAVEGAME).exists()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn savegame_exists() -> bool {
        stdweb::web::window().local_storage().contains_key(SAVEGAME)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn delete_savegame() {
        std::fs::remove_file(std::path::Path::new(SAVEGAME)).expect("Failed to delete savegame");
    }

    #[cfg(target_arch = "wasm32")]
    fn delete_savegame() {
        stdweb::web::window().local_storage().remove(SAVEGAME);
    }
}

impl<'a> System<'a> for LoadSystem {
    type SystemData = LoadSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        assert!(Self::savegame_exists());
        let reader = Self::reader();
        let mut ron_data = String::new();
        flate2::read::GzDecoder::new(reader)
            .read_to_string(&mut ron_data)
            .expect("Failed to decompress savegame");

        let deserializer =
            ron::Deserializer::from_str(&ron_data).expect("Failed to create deserializer");
        data.deser(deserializer);

        // Load resources from the serialization helper
        let serialization_helper = &(data.components.0).0;
        assert_eq!((serialization_helper,).join().count(), 1);
        let resources: &SerializationHelper = (serialization_helper,).join().next().unwrap().0;
        *data.map = resources.map.clone();
        *data.game_log = resources.game_log.clone();

        // Clean up serialization helper entities
        for (entity, _) in (&data.entities, serialization_helper).join() {
            data.entities
                .delete(entity)
                .expect("Failed to clean up serialization helper");
        }

        // We're a roguelike!
        Self::delete_savegame();
    }
}
