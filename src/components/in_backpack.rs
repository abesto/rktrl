use legion::Entity;

#[derive(PartialEq, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

impl InBackpack {
    #[must_use]
    pub fn new(owner: Entity) -> InBackpack {
        InBackpack { owner }
    }
}
