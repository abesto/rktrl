use legion::{system, systems::CommandBuffer, Entity};

use crate::components::*;
use crate::resources::*;

#[system(for_each)]
pub fn damage(
    entity: &Entity,
    stats: &mut CombatStats,
    damage: &SufferDamage,
    position: &Position,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    stats.hp -= damage.amount.iter().sum::<i32>();
    map.add_bloodstain(*position);
    // TODO maybe there's a more efficient way of flushing all components of a type
    //      or maybe a component is not the right way to model this
    //      (but instead some signal / queue or w/e)
    commands.remove_component::<SufferDamage>(*entity);
}
