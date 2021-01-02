use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Default, Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "df6ad6f6-e873-4b2b-a332-dc6ef411f7d5"]
pub struct SingleActivation;
