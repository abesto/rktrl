use std::convert::TryInto;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use bracket_lib::prelude::Point;
use macro_attr::*;
use newtype_derive::*;
use specs::prelude::*;
use specs_derive::Component;

use crate::lib::vector::Vector;

macro_attr! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash,
             Component,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!)]
    pub struct Position(Point);
}

impl Position {
    #[inline]
    #[must_use]
    pub fn new<T>(x: T, y: T) -> Position
        where
            T: TryInto<i32>,
    {
        Point::new(x, y).into()
    }

    pub fn distance(&self, to: Position) -> f32 {
        (*self - to).len()
    }
}

impl From<(i32, i32)> for Position {
    fn from(pair: (i32, i32)) -> Self {
        Point::from_tuple(pair).into()
    }
}

impl_op_ex!(-|lhs: Position, rhs: Position| -> Vector { (*lhs - *rhs).into() });
impl_op_ex_commutative!(+ |a: Position, b: Vector| -> Position { (*a + *b).into() });
