use crate::components::Player;
use legion::{component, storage::Component, world::SubWorld, Entity, EntityStore, IntoQuery};

pub trait WorldExt {
    fn has_component<T: Component>(&self, entity: Entity) -> bool;
    fn get_component<T: Component + Clone>(&self, entity: Entity) -> T;
    fn player_entity(&self) -> &Entity;
    fn player_component<T: Component + Clone>(&self) -> T;
}

impl<'a> WorldExt for SubWorld<'a> {
    fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if let Ok(entry) = self.entry_ref(entity) {
            entry.archetype().layout().has_component::<T>()
        } else {
            false
        }
    }

    /// Explodes if the entity doesn't have the component
    fn get_component<T: Component + Clone>(&self, entity: Entity) -> T {
        self.entry_ref(entity)
            .unwrap()
            .get_component::<T>()
            .unwrap()
            .clone()
    }

    fn player_entity(&self) -> &Entity {
        let entities: Vec<_> = <(Entity,)>::query()
            .filter(component::<Player>())
            .iter(self)
            .collect();
        assert_eq!(entities.len(), 1);
        entities[0].0
    }

    fn player_component<T: Component + Clone>(&self) -> T {
        self.get_component(*self.player_entity())
    }
}
