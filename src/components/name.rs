use legion_typeuuid::register_serialize;
use macro_attr::*;
use newtype_derive::*;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             Serialize, Deserialize, TypeUuid,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!, NewtypeDisplay!)]
    #[uuid = "6137cc44-1a58-4c61-81e1-ec497bf7baee"]
    pub struct Name(String);
}
register_serialize!(Name);
