use shipyard::EntityId;

#[derive(PartialEq, Debug, Clone)]
pub struct InBackpack {
    pub owner: EntityId,
}

impl InBackpack {
    pub fn new(owner: EntityId) -> InBackpack {
        InBackpack { owner }
    }
}
