use std::cmp::{max, min};

use bracket_lib::prelude::*;

use crate::{components::position::Position, util::vector::Vector};

pub trait RectExt {
    fn p1(&self) -> Position;
    fn p2(&self) -> Position;
    fn top_left(&self) -> Position;
    fn top_right(&self) -> Position;
    fn bottom_left(&self) -> Position;
    fn bottom_right(&self) -> Position;
    fn interior(&self) -> Rect;
    fn position(&self, offset: Vector) -> Position;
}

impl RectExt for Rect {
    fn p1(&self) -> Position {
        Position::new(self.x1, self.y1)
    }

    fn p2(&self) -> Position {
        Position::new(self.x2, self.y2)
    }

    fn top_left(&self) -> Position {
        self.p1()
    }

    fn top_right(&self) -> Position {
        Position::new(self.x2, self.y1)
    }

    fn bottom_left(&self) -> Position {
        Position::new(self.x1, self.y2)
    }

    fn bottom_right(&self) -> Position {
        self.p2()
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

    fn position(&self, offset: Vector) -> Position {
        assert!(offset.x.abs() < self.width());
        assert!(offset.y.abs() < self.height());
        let basis = Position::new(
            if offset.x < 0 { self.x2 + 1 } else { self.x1 },
            if offset.y < 0 { self.y2 + 1 } else { self.y1 },
        );
        basis + offset
    }
}
