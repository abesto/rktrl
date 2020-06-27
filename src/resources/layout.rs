use std::convert::TryInto;

use bracket_lib::prelude::Rect;

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

    pub fn inventory<T>(&self, item_count: T) -> Rect
    where
        T: TryInto<i32>,
    {
        let width_third = self.width / 3;
        let item_count_i32: i32 = item_count.try_into().ok().unwrap();
        let height = item_count_i32 + 3;
        Rect::with_size(width_third, (self.height + height) / 2, width_third, height)
    }
}
