use std::cmp::{max, min};
use std::convert::TryInto;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use bracket_lib::prelude::{Point, Rect};
use macro_attr::*;
use newtype_derive::*;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::Rng;

use crate::lib::vector::Vector;

macro_attr! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash,
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

pub trait RectExt {
    fn p1(&self) -> Position;
    fn p2(&self) -> Position;
    fn interior(&self) -> Rect;
}

impl RectExt for Rect {
    fn p1(&self) -> Position {
        Position::new(self.x1, self.y1)
    }

    fn p2(&self) -> Position {
        Position::new(self.x2, self.y2)
    }

    fn interior(&self) -> Rect {
        let smaller = Rect::with_exact(self.x1 + 1, self.y1 + 1, self.x2 - 1, self.y2 - 1);
        Rect::with_exact(
            min(smaller.x1, smaller.x2),
            min(smaller.y1, smaller.y2),
            max(smaller.x1, smaller.x2),
            max(smaller.y1, smaller.y2),
        )
    }
}
