use std::cmp::{max, min};
use std::convert::TryInto;

use bracket_lib::prelude::{Point, Rect};

#[derive(Copy, Clone, Debug)]
pub struct Layout {
    pub width: i32,
    pub height: i32,
    pub panel_height: i32,
}

impl Layout {
    pub fn map(&self) -> Rect {
        Rect::with_size(0, 0, self.width, self.height - self.panel_height)
    }

    pub fn panel(&self) -> Rect {
        Rect::with_size(
            0,
            self.height - self.panel_height,
            self.width,
            self.panel_height,
        )
    }

    pub fn inventory<T>(&self, item_count: T, max_item_len: T) -> Rect
        where
            T: TryInto<i32>,
    {
        let item_count_i32: i32 = item_count.try_into().ok().unwrap();
        let max_item_len_i32: i32 = max_item_len.try_into().ok().unwrap();

        // +7 width: 2 border + 2 padding + 4 shortcut - 1 bracket_lib issue 96 workaround
        let width = min(self.width, max(self.width / 3, max_item_len_i32 + 7));
        // +3 height: 2 border + 2 padding - 1 bracket_lib issue 96 workaround
        let height = item_count_i32 + 3;
        Rect::with_size(
            (self.width - width) / 2,
            (self.height - height) / 2,
            width,
            height,
        )
    }

    pub fn hunger_status(&self, length: i32) -> Point {
        Point::new(self.width - length - 1, self.panel().y1 - 1)
    }
}
