use crate::lib::vector::Vector;
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use specs::{Component, VecStorage};
use specs_derive::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[storage(VecStorage)]
pub struct Position {
    pub vector: Vector,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Vector::new(x, y).into()
    }

    pub fn x(&self) -> i32 {
        self.vector.x
    }

    pub fn y(&self) -> i32 {
        self.vector.y
    }
}

impl From<Vector> for Position {
    fn from(vector: Vector) -> Self {
        Position { vector }
    }
}

impl_op_ex_commutative!(+ |lhs: &Position, rhs: &Vector| -> Position
    { Position { vector: lhs.vector + rhs } }
);

impl_op_ex!(+= |lhs: &mut Position, rhs: &Vector| {
    lhs.vector += rhs;
});
