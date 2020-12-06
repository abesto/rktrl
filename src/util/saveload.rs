use std::fs::File;

use lazy_static::lazy_static;
use legion::{component, Resources, World};
use legion_typeuuid::{collect_registry, SerializableTypeUuid};
use serde::{de::DeserializeSeed, Deserialize, Serialize};

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

const SAVEGAME: &str = "./savegame.bincode.gz";
lazy_static! {
    static ref BINCODE_OPTIONS: bincode::DefaultOptions = bincode::DefaultOptions::default();
    static ref REGISTRY: legion::Registry<SerializableTypeUuid> = collect_registry();
}

fn writer() -> File {
    File::create(SAVEGAME).expect("Failed to create file")
}

fn reader() -> File {
    File::open(SAVEGAME).expect("Failed to open file")
}

fn delete_savegame() {
    std::fs::remove_file(std::path::Path::new(SAVEGAME)).expect("Failed to delete savegame");
}

pub fn savegame_exists() -> bool {
    std::path::Path::new(SAVEGAME).exists()
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
    // Open up the save file
    let writer = writer();
    let encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::fast());
    let mut ser = bincode::Serializer::new(encoder, *BINCODE_OPTIONS);

    // Serialize entities, components
    world
        .as_serializable(component::<SerializeMe>(), &*REGISTRY)
        .serialize(&mut ser)
        .unwrap();

    // Serialize resources
    foreach_resource!(ser.serialize_resource::<R>(resources));
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
    delete_savegame();
}
