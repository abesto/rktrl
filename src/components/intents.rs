// TODO remove this file once migration to CAE is done
use legion::Entity;
use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::components::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "2730588d-3760-484c-8f0c-1ac4f5428006"]
pub struct MeleeIntent {
    pub target: Entity,
}
register_serialize!(MeleeIntent);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "b8483f51-556f-4d71-9bcb-805c986d79e6"]
pub struct PickupIntent {
    pub item: Entity,
}
register_serialize!(PickupIntent);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "a8ebdcdb-72a5-4165-b40d-6e852c495147"]
pub enum UseTarget {
    SelfCast,
    Position(Position),
}
register_serialize!(UseTarget);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "fd7bb480-2d6c-4a86-ab70-8675e4b08b3a"]
pub struct UseIntent {
    pub item: Entity,
    pub target: UseTarget,
}
register_serialize!(UseIntent);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "8e3fb737-2a8f-4461-b550-277c448430a4"]
pub struct DropIntent {
    pub item: Entity,
}
register_serialize!(DropIntent);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypeUuid)]
#[uuid = "bda91207-a6d2-4284-9e0d-b7c10adfc16d"]
pub struct RemoveIntent {
    pub item: Entity,
}
register_serialize!(RemoveIntent);
