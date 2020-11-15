use macro_attr::*;
use newtype_derive::*;

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!, NewtypeDisplay!)]
    pub struct Name(String);
}
