use crate::lib::vector::Vector;
use shipyard::EntityId;

#[derive(Debug, Clone)]
pub struct MeleeIntent {
    pub target: EntityId,
}

#[derive(Debug, Clone)]
pub struct PickUpIntent {
    pub item: EntityId,
}

#[derive(Debug, Clone)]
pub struct UseIntent {
    pub item: EntityId,
}

#[derive(Debug, Clone)]
pub struct DropIntent {
    pub item: EntityId,
}

#[derive(Debug, Clone)]
pub struct MoveIntent {
    pub vector: Vector,
}
