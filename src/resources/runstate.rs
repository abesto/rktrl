use std::collections::VecDeque;
use std::fmt;

use macro_attr::*;
use newtype_derive::*;
use specs::prelude::Entity;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, Copy, Clone, Debug, EnumIter)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

impl Default for MainMenuSelection {
    fn default() -> Self {
        MainMenuSelection::NewGame
    }
}

impl MainMenuSelection {
    pub fn down(self) -> MainMenuSelection {
        MainMenuSelection::iter()
            .skip_while(|s| s != &self)
            .nth(1)
            .unwrap_or_else(|| MainMenuSelection::iter().next().unwrap())
    }

    pub fn up(self) -> MainMenuSelection {
        MainMenuSelection::iter()
            .rev()
            .skip_while(|s| s != &self)
            .nth(1)
            .unwrap_or_else(|| MainMenuSelection::iter().last().unwrap())
    }
}

impl fmt::Display for MainMenuSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MainMenuSelection::*;
        write!(
            f,
            "{}",
            match self {
                NewGame => "Begin New Game",
                LoadGame => "Load Game",
                Quit => "Quit",
            }
        )
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { selection: MainMenuSelection },
}

impl RunState {
    #[must_use]
    pub fn show_inventory(self) -> bool {
        self == RunState::ShowDropItem || self == RunState::ShowInventory
    }
}

impl Default for RunState {
    fn default() -> Self {
        RunState::MainMenu {
            selection: MainMenuSelection::default(),
        }
    }
}

macro_attr! {
    #[derive(Clone, PartialEq, Default,
             NewtypeDebug!, NewtypeDeref!, NewtypeDerefMut!, NewtypeFrom!)]
    pub struct RunStateQueue(VecDeque<RunState>);
}
