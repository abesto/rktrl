use std::convert::TryInto;

use bracket_lib::prelude::Point;
use macro_attr::*;
use newtype_derive::*;
use specs::prelude::*;
use specs_derive::Component;

macro_attr! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash,
             Component,
             NewtypeDebug!,
             NewtypeAdd!(Point), NewtypeDeref!, NewtypeFrom!)]
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
}

impl From<(i32, i32)> for Position {
    fn from(pair: (i32, i32)) -> Self {
        Point::from_tuple(pair).into()
    }
}
