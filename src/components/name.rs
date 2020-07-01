use macro_attr::*;
use newtype_derive::*;
use serde::{Deserialize, Serialize};
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};

macro_attr! {
    #[derive(Clone, PartialEq, Eq, Hash,
             Component, ConvertSaveload,
             NewtypeDebug!, NewtypeDeref!, NewtypeFrom!, NewtypeDisplay!)]
    pub struct Name(String);
}
