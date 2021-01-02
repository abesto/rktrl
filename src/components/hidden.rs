use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "637cef29-2dd3-40e0-a0ac-89346d9fa7f1"]
pub struct Hidden;
