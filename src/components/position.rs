use crate::lib::vector::Vector;
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use bracket_lib::prelude::Point;
use specs::{Component, VecStorage};
use specs_derive::Component;
use std::convert::TryInto;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[storage(VecStorage)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Position {
        Position { x, y }
    }
}

impl From<Vector> for Position {
    fn from(vector: Vector) -> Self {
        Position {
            x: vector.x.try_into().unwrap(),
            y: vector.y.try_into().unwrap(),
        }
    }
}

impl From<(u16, u16)> for Position {
    fn from(pair: (u16, u16)) -> Self {
        Position {
            x: pair.0,
            y: pair.1,
        }
    }
}

impl From<&Position> for Vector {
    fn from(position: &Position) -> Self {
        Vector {
            x: position.x.into(),
            y: position.y.into(),
        }
    }
}

impl std::convert::TryFrom<&Point> for Position {
    type Error = std::num::TryFromIntError;

    fn try_from(value: &Point) -> Result<Self, Self::Error> {
        value
            .x
            .try_into()
            .and_then(|x| value.y.try_into().map(|y| Position { x, y }))
    }
}

impl_op_ex_commutative!(+ |lhs: &Position, rhs: &Vector| -> Position {
    let lhs_vector: Vector = lhs.into();
    (lhs_vector + rhs).into()
});

impl_op_ex!(+= |lhs: &mut Position, rhs: &Vector| {
    let sum = *lhs + rhs;
    lhs.x = sum.x;
    lhs.y = sum.y;
});
