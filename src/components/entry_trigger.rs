use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "eebb7c2f-0c31-479a-b130-be018887ba63"]
pub struct EntryTrigger;
