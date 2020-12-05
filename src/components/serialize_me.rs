use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "5209ef10-8086-432b-9a24-e1500a25aace"]
pub struct SerializeMe;
register_serialize!(SerializeMe);
