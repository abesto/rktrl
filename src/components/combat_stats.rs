#[derive(Debug, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

impl CombatStats {
    pub fn with_hp(&self, new_hp: i32) -> CombatStats {
        let mut new = self.clone();
        new.hp = new_hp;
        new
    }
}

#[derive(Clone)]
pub struct MeleePowerBonus {
    pub power: i32,
}

impl MeleePowerBonus {
    #[must_use]
    pub fn new(power: i32) -> Self {
        MeleePowerBonus { power }
    }
}

#[derive(Clone)]
pub struct DefenseBonus {
    pub defense: i32,
}

impl DefenseBonus {
    #[must_use]
    pub fn new(defense: i32) -> Self {
        DefenseBonus { defense }
    }
}
