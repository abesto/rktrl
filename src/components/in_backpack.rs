use legion::Entity;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "a7418e38-6ec8-4c6a-a4da-99b31589bafb"]
pub struct InBackpack {
    pub owner: Entity,
}

impl InBackpack {
    #[must_use]
    pub fn new(owner: Entity) -> InBackpack {
        InBackpack { owner }
    }
}
