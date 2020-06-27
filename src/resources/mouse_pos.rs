use bracket_lib::prelude::*;

pub struct MousePos {
    pub point: Point,
}

impl MousePos {
    #[must_use]
    pub const fn new(point: Point) -> MousePos {
        MousePos { point }
    }
}

impl Default for MousePos {
    fn default() -> Self {
        MousePos::new(Point::new(0, 0))
    }
}
