pub struct Rect {
    pub x1: u16,
    pub x2: u16,
    pub y1: u16,
    pub y2: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (u16, u16) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}
