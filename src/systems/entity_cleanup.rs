/// Downstream systems might need to read components from entities that, semantically, an
/// upstream system deletes (eg. game_log needs to read the Name of entities to report their death).
/// This system implements a delayed, controlled removal of entities.
use crate::systems::prelude::*;
use crossbeam_queue::SegQueue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityCleanupRequest {
    entity: Entity,
}

impl EntityCleanupRequest {
    #[must_use]
    pub fn new(entity: Entity) -> EntityCleanupRequest {
        EntityCleanupRequest { entity }
    }
}

pub type DeferredCleanup = SegQueue<EntityCleanupRequest>;

pub trait DeferredCleanupHelper {
    fn entity(&mut self, entity: Entity);
}

impl DeferredCleanupHelper for DeferredCleanup {
    fn entity(&mut self, entity: Entity) {
        self.push(EntityCleanupRequest::new(entity));
    }
}

#[system]
pub fn entity_cleanup(
    #[resource] cleanup_requests: &mut DeferredCleanup,
    commands: &mut CommandBuffer,
) {
    while let Some(request) = cleanup_requests.pop() {
        commands.remove(request.entity);
    }
}
