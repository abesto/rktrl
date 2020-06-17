use std::convert::TryInto;

use bracket_lib::prelude::Point;
use macro_attr::*;
use newtype_derive::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

    pub fn len(&self) -> f64 {
        let xf: f64 = self.x.into();
        let yf: f64 = self.y.into();
        (xf.powf(2.0) + yf.powf(2.0)).sqrt()
    }

    pub fn unit(heading: Heading) -> Vector {
        match heading {
            Heading::North => Vector::new(0, -1),
            Heading::East => Vector::new(1, 0),
            Heading::South => Vector::new(0, 1),
            Heading::West => Vector::new(-1, 0),
        }
    }
}
