use specs::prelude::*;
use specs_derive::Component;

#[derive(PartialEq, Component, Debug, Clone)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: String) -> Name {
        Name { name }
    }
}
