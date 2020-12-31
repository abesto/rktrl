use legion::{system, world::SubWorld, IntoQuery, Resources};

use crate::{
    cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link},
    components::*,
    resources::*,
};

cae_system_state!(DamageSystemState {
    damage(link) { matches!(link.label, Label::Damage {..}) }
});

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
        let (amount, target, bleeding) = match damage.label {
            Label::Damage {
                amount,
                to,
                bleeding,
            } => (amount, to, bleeding),
            _ => unreachable!(),
        };

        if amount <= 0 {
            continue;
        }

        let (stats, position) = <(&mut CombatStats, &Position)>::query()
            .get_mut(world, target)
            .unwrap();
        stats.hp -= amount;

        if bleeding {
            map.add_bloodstain(*position);
        }

        if stats.hp <= 0 {
            cae.add_effect(&damage, Label::Death { entity: target });
        }
    }
}
