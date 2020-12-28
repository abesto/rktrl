use legion::{system, world::SubWorld, IntoQuery, Resources};

use crate::{
    cause_and_effect::{CAESubscription, CauseAndEffect, Label, Link},
    components::*,
    resources::*,
};

pub struct DamageSystemState {
    subscription: CAESubscription,
}

impl DamageSystemState {
    fn subscription_filter(link: &Link) -> bool {
        matches!(link.label, Label::Damage { .. })
    }

    pub fn new(resources: &Resources) -> DamageSystemState {
        let cae = &mut *resources.get_mut::<CauseAndEffect>().unwrap();
        DamageSystemState {
            subscription: cae.subscribe(DamageSystemState::subscription_filter),
        }
    }
}

#[system]
#[read_component(Position)]
#[write_component(CombatStats)]
pub fn damage(
    #[state] state: &DamageSystemState,
    #[resource] map: &mut Map,
    #[resource] cae: &mut CauseAndEffect,
    world: &mut SubWorld,
) {
    for damage in cae.get_queue(state.subscription) {
        let (amount, target) = match damage.label {
            Label::Damage { amount, to } => (amount, to),
            _ => unreachable!(),
        };

        let (stats, position) = <(&mut CombatStats, &Position)>::query()
            .get_mut(world, target)
            .unwrap();
        stats.hp -= amount;
        map.add_bloodstain(*position);
    }
}
