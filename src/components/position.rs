use std::convert::TryInto;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use bracket_lib::prelude::{Point, Rect};
use legion_typeuuid::register_serialize;
use macro_attr::*;
use newtype_derive::*;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::Rng;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::util::vector::Vector;

macro_attr! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash,
             Serialize, Deserialize, TypeUuid,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!)]
    #[uuid = "230413cb-9c33-4922-8559-c4aa4c144916"]
    pub struct Position(Point);
}
register_serialize!(Position);

impl Position {
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

// Pick a random point from a rect

pub struct PositionSampler {
    bounds: Rect,
}

impl UniformSampler for PositionSampler {
    type X = Position;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let lo = low.borrow();
        let hi = high.borrow();
        PositionSampler {
            bounds: Rect::with_exact(lo.x, lo.y, hi.x, hi.y),
        }
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let lo = low.borrow();
        let hi = high.borrow();
        PositionSampler {
            bounds: Rect::with_exact(lo.x, lo.y, hi.x + 1, hi.y + 1),
        }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Position::new(
            rng.gen_range(self.bounds.x1, self.bounds.x2),
            rng.gen_range(self.bounds.y1, self.bounds.y2),
        )
    }
}

impl SampleUniform for Position {
    type Sampler = PositionSampler;
}
