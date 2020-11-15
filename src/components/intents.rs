use crate::components::position::Position;
use legion::Entity;

#[derive(Debug, Clone)]
pub struct MeleeIntent {
    pub target: Entity,
}

#[derive(Debug, Clone)]
pub struct PickupIntent {
    pub item: Entity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseTarget {
    SelfCast,
    Position(Position),
}

#[derive(Debug, Clone)]
pub struct UseIntent {
    pub item: Entity,
    pub target: UseTarget,
}

#[derive(Debug, Clone)]
pub struct DropIntent {
    pub item: Entity,
}

#[derive(Debug, Clone)]
pub struct RemoveIntent {
    pub item: Entity,
}
