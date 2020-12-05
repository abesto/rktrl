use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "b65a3135-2d2d-445e-8b5e-ce50f9e58dfa"]
pub struct Item;
register_serialize!(Item);
