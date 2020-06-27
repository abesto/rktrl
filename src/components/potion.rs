#[derive(PartialEq, Debug, Clone)]
pub struct Potion {
    pub heal_amount: i32,
}

impl Potion {
    pub const fn new(heal_amount: i32) -> Potion {
        Potion { heal_amount }
    }
}
