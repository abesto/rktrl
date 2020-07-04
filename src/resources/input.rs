use bracket_lib::prelude::*;

#[derive(Debug)]
pub struct Input {
    pub key: Option<VirtualKeyCode>,
    pub shift: bool,
    pub mouse_pos: Point,
    pub left_click: bool,
}

impl From<&BTerm> for Input {
    fn from(term: &BTerm) -> Self {
        Input {
            key: term.key,
            shift: term.shift,
            mouse_pos: term.mouse_point(),
            left_click: term.left_click,
        }
    }
}
