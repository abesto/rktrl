use std::convert::TryInto;

use bracket_lib::prelude::{DistanceAlg, Point};
use macro_attr::*;
use newtype_derive::*;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Heading {
    North,
    East,
    South,
    West,
}

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             NewtypeDebug!, NewtypeAdd!(Point), NewtypeDeref!, NewtypeFrom!)]
    pub struct Vector(Point);
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

    // This can be a const fn once RFC 2342 is merged
    pub fn unit(heading: Heading) -> Vector {
        match heading {
            Heading::North => Vector::new(0, -1),
            Heading::East => Vector::new(1, 0),
            Heading::South => Vector::new(0, 1),
            Heading::West => Vector::new(-1, 0),
        }
    }
}
