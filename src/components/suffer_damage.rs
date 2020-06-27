use shipyard::*;

#[derive(Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(
        entities: &EntitiesView,
        suffer_damages: &mut ViewMut<SufferDamage>,
        victim: EntityId,
        amount: i32,
    ) {
        if let Ok(suffering) = (suffer_damages).get(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            entities.add_component(suffer_damages, dmg, victim);
        }
    }
}
