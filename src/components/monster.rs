use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug)]
pub struct Monster {}

impl Monster {
    pub fn new() -> Monster {
        Monster {}
    }
}
