use legion::{Entity, systems::CommandBuffer};
use legion_typeuuid::register_serialize;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "46fbedf3-f32b-485e-9422-b83bbc8b8c84"]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}
register_serialize!(SufferDamage);

impl SufferDamage {
    pub fn new_damage(commands: &mut CommandBuffer, entity: Entity, amount: i32) {
        commands.exec_mut(move |world| {
            let mut entry = world.entry(entity).unwrap();
            if !entry.archetype().layout().has_component::<SufferDamage>() {
                entry.add_component(SufferDamage { amount: vec![] });
            }
            entry
                .get_component_mut::<SufferDamage>()
                .unwrap()
                .amount
                .push(amount);
        });
    }
}
