use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "feff948e-2e0f-45ef-bd14-1efe401fe08a"]
pub struct Player;
