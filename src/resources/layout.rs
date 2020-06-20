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
}
