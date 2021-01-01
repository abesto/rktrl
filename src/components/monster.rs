use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "d72582fd-e492-4665-9d0d-7842b7029ef9"]
pub struct Monster;
