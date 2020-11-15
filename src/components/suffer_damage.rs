use legion::{systems::CommandBuffer, Entity};

#[derive(Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

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
