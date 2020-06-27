use bracket_lib::prelude::*;

pub struct Input {
    pub key: Option<VirtualKeyCode>,
    pub mouse_pos: Point,
    pub left_click: bool,
}

impl From<&BTerm> for Input {
    fn from(term: &BTerm) -> Self {
        Input {
            key: term.key,
            mouse_pos: term.mouse_point(),
            left_click: term.left_click,
        }
    }
}
