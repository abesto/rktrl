use std::io::Write;

use lazy_static::lazy_static;
use legion::{component, Resources, World};
use legion_typeuuid::SerializableTypeUuid;
use serde::{de::DeserializeSeed, Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(target_arch = "wasm32")]
use std::io::{Cursor, Error, ErrorKind, Result as IOResult};

use crate::components::SerializeMe;
use crate::resources::{GameLog, Map};

/// Execute code against each resource type we want to serialize, in a stable order.
/// Used to guarantee serialization and deserialization use the same order.
macro_rules! foreach_resource {
    ($obj:ident.$f:ident::<R>($arg:ident)) => {
        $obj.$f::<Map>($arg);
        $obj.$f::<GameLog>($arg);
    };
}

#[cfg(not(target_arch = "wasm32"))]
const SAVEGAME: &str = "./savegame.bincode.gz";
#[cfg(target_arch = "wasm32")]
const SAVEGAME: &str = "savegame";

#[cfg(target_arch = "wasm32")]
pub struct LocalStorageWriter {
    buffer: Vec<u8>,
    storage: web_sys::Storage,
}

#[cfg(target_arch = "wasm32")]
impl LocalStorageWriter {
    fn new() -> LocalStorageWriter {
        LocalStorageWriter {
            buffer: vec![],
            storage: web_sys::window().unwrap().local_storage().unwrap().unwrap(),
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
            .set_item(SAVEGAME, &encoded)
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to write into local storage"))
    }
}

// TODO The `inventory` and `linkme` crates don't support WASM, so we must explicitly list
//      all components. See https://github.com/rustwasm/wasm-bindgen/issues/1216
macro_rules! register_components {
    ($registry:ident, $($component:ident),+) => {
        $(
            $registry.register_auto_mapped::<$component>();
        )+
    }
}

lazy_static! {
    static ref BINCODE_OPTIONS: bincode::DefaultOptions = bincode::DefaultOptions::default();
    static ref REGISTRY: legion::Registry<SerializableTypeUuid> = {
        use crate::components::*;
        let mut registry = legion::Registry::default();
        register_components!(
            registry,
            AreaOfEffect,
            BlocksTile,
            CombatStats,
            Confusion,
            Consumable,
            DefenseBonus,
            Equippable,
            Equipped,
            HungerClock,
            InBackpack,
            InflictsDamage,
            Item,
            MeleePowerBonus,
            Monster,
            Name,
            Player,
            Position,
            ProvidesFood,
            ProvidesHealing,
            Ranged,
            Renderable,
            SerializeMe,
            Viewshed
        );
        registry
    };
}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> web_sys::Storage {
    web_sys::window().unwrap().local_storage().unwrap().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn writer() -> File {
    File::create(SAVEGAME).expect("Failed to create file")
}

#[cfg(target_arch = "wasm32")]
fn writer() -> LocalStorageWriter {
    LocalStorageWriter::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn reader() -> File {
    File::open(SAVEGAME).expect("Failed to open file")
}

#[cfg(target_arch = "wasm32")]
fn reader() -> Cursor<Vec<u8>> {
    let storage = local_storage();
    let encoded = storage
        .get(SAVEGAME)
        .expect("Failed to read from local storage")
        .unwrap()
        .into_bytes();
    Cursor::new(base64::decode(encoded).expect("Failed to base64 decode savegame"))
}

#[cfg(not(target_arch = "wasm32"))]
fn delete_savegame() {
    std::fs::remove_file(std::path::Path::new(SAVEGAME)).expect("Failed to delete savegame");
}

#[cfg(target_arch = "wasm32")]
fn delete_savegame() {
    local_storage().remove_item(SAVEGAME).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn savegame_exists() -> bool {
    std::path::Path::new(SAVEGAME).exists()
}

#[cfg(target_arch = "wasm32")]
pub fn savegame_exists() -> bool {
    local_storage().get(SAVEGAME).unwrap().is_some()
}

trait SerializeResource {
    fn serialize_resource<T: 'static + Serialize>(&mut self, resources: &Resources);
}

impl<W: std::io::Write, O: bincode::Options> SerializeResource for bincode::Serializer<W, O> {
    fn serialize_resource<T: 'static + Serialize>(&mut self, resources: &Resources) {
        resources.get::<T>().unwrap().serialize(self).unwrap();
    }
}

pub fn save(world: &World, resources: &Resources) {
    let mut writer = writer();

    {
        let mut encoder = flate2::write::GzEncoder::new(&mut writer, flate2::Compression::fast());
        let mut ser = bincode::Serializer::new(&mut encoder, *BINCODE_OPTIONS);

        // Serialize entities, components
        world
            .as_serializable(component::<SerializeMe>(), &*REGISTRY)
            .serialize(&mut ser)
            .unwrap();

        // Serialize resources
        foreach_resource!(ser.serialize_resource::<R>(resources));
    }

    writer.flush().unwrap();
}

trait DeserializeResource<'de> {
    fn deserialize_resource<T: 'static + Deserialize<'de>>(&mut self, resources: &mut Resources);
}

impl<'de, R: bincode::BincodeRead<'de>, O: bincode::Options> DeserializeResource<'de>
    for bincode::Deserializer<R, O>
{
    fn deserialize_resource<T: 'static + Deserialize<'de>>(&mut self, resources: &mut Resources) {
        resources.remove::<T>();
        resources.insert(T::deserialize(self).unwrap());
    }
}

pub fn load(world: &mut World, resources: &mut Resources) {
    // Open up the save
    let reader = reader();
    let mut decoder = flate2::read::GzDecoder::new(reader);
    let mut deser = bincode::Deserializer::with_reader(&mut decoder, *BINCODE_OPTIONS);

    // Load entities / components
    world.clear();
    REGISTRY
        .as_deserialize_into_world(world)
        .deserialize(&mut deser)
        .unwrap();

    // Load resources
    foreach_resource!(deser.deserialize_resource::<R>(resources));

    // We're a roguelike!
    // TODO make sure to not lose the last save if the game is closed / crashes
    delete_savegame();
}
