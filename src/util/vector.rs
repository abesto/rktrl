use std::convert::TryInto;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use bracket_lib::prelude::{DistanceAlg, Point};
use macro_attr::*;
use newtype_derive::*;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Hash, Clone, Copy, EnumIter, Debug)]
pub enum Heading {
    North,
    East,
    South,
    West,
}

macro_attr! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash,
             NewtypeDebug!, NewtypeAdd!, NewtypeDeref!, NewtypeFrom!)]
    pub struct Vector(Point);
}

impl Heading {
    pub fn rotate_cw(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub fn rotate_ccw(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }
}

impl Vector {
    #[inline]
    #[must_use]
    pub fn new<T>(x: T, y: T) -> Vector
    where
        T: TryInto<i32>,
    {
        Point::new(x, y).into()
    }

    pub const fn constant(x: i32, y: i32) -> Self {
        Vector(Point::constant(x, y))
    }

    pub fn len(&self) -> f32 {
        DistanceAlg::Pythagoras.distance2d(Point::constant(0, 0), **self)
    }

    pub const fn unit(heading: Heading) -> Vector {
        match heading {
            Heading::North => Vector::constant(0, -1),
            Heading::East => Vector::constant(1, 0),
            Heading::South => Vector::constant(0, 1),
            Heading::West => Vector::constant(-1, 0),
        }
    }

    pub fn rotated(&self) -> Vector {
        /* Rotate by 90 degrees clockwise */
        Vector::constant(self.y, -self.x)
    }
}

impl From<Heading> for Vector {
    fn from(heading: Heading) -> Self {
        Vector::unit(heading)
    }
}

impl_op_ex_commutative!(+ |a: Vector, b: Heading| -> Vector { a + Vector::from(b) });
impl_op_ex!(+ |a: Heading, b: Heading| -> Vector { Vector::from(a) + b });
