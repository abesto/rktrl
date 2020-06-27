use macro_attr::*;
use newtype_derive::*;
use specs::prelude::*;
use specs_derive::Component;

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             Component,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!, NewtypeDisplay!)]
    pub struct Name(String);
}
