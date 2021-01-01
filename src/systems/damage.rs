use crate::systems::prelude::*;

cae_system_state!(DamageSystemState { damage: Damage });

#[system]
#[read_component(Position)]
#[write_component(CombatStats)]
pub fn damage(
    #[state] state: &DamageSystemState,
    #[resource] map: &mut Map,
    #[resource] cae: &mut CauseAndEffect,
    world: &mut SubWorld,
) {
    for damage in cae.get_queue(state.damage) {
        extract_label!(damage @ Damage => amount, to, bleeding);

        if amount <= 0 {
            continue;
        }

        let (stats, position) = <(&mut CombatStats, &Position)>::query()
            .get_mut(world, to)
            .unwrap();
        stats.hp -= amount;

        if bleeding {
            map.add_bloodstain(*position);
        }

        if stats.hp <= 0 {
            cae.add_effect(&damage, Label::Death { entity: to });
        }
    }
}
