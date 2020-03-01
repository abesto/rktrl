use bracket_lib::prelude::VirtualKeyCode;

pub struct Input {
    pub key: Option<VirtualKeyCode>,
}

impl Default for Input {
    fn default() -> Self {
        Input { key: Option::None }
    }
}

impl Input {
    pub fn key(key: Option<VirtualKeyCode>) -> Input {
        Input { key }
    }
}
