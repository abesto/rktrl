use legion::{component, storage::Component, world::SubWorld, Entity, EntityStore, IntoQuery};

use crate::components::Player;

pub trait WorldExt {
    fn has_component<T: Component>(&self, entity: Entity) -> bool;
    fn get_component<T: Component + Clone>(&self, entity: Entity) -> T;
    fn maybe_player_entity(&self) -> Option<&Entity>;
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
        // TODO this would be more rustacean (?) if it returned an Option<T>
        self.entry_ref(entity)
            .unwrap()
            .get_component::<T>()
            .unwrap()
            .clone()
    }

    fn maybe_player_entity(&self) -> Option<&Entity> {
        <(Entity,)>::query()
            .filter(component::<Player>())
            .iter(self)
            .map(|x| x.0)
            .next()
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
