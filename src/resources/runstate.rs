use std::collections::VecDeque;
use std::fmt;

use macro_attr::*;
use newtype_derive::*;
use specs::prelude::Entity;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::systems::saveload::LoadSystem;

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
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        selection: MainMenuSelection,
        load_enabled: bool,
    },
    SaveGame,
    LoadGame,
    NextLevel,
}

impl RunState {
    #[must_use]
    pub fn show_inventory(self) -> bool {
        self == RunState::ShowDropItem || self == RunState::ShowInventory
    }

    pub fn main_menu_item_enabled(&self, item: MainMenuSelection) -> bool {
        match self {
            RunState::MainMenu { load_enabled, .. } => match item {
                MainMenuSelection::LoadGame => *load_enabled,
                _ => true,
            },
            _ => unimplemented!(),
        }
    }

    fn main_menu_next_enabled<F>(&self, next_fn: F) -> MainMenuSelection
    where
        F: Fn(MainMenuSelection) -> MainMenuSelection,
    {
        match self {
            RunState::MainMenu { selection, .. } => {
                let mut candidate = next_fn(*selection);
                while !self.main_menu_item_enabled(candidate) && &candidate != selection {
                    candidate = next_fn(candidate);
                }
                assert!(self.main_menu_item_enabled(candidate));
                candidate
            }
            _ => unimplemented!(),
        }
    }

    pub fn main_menu_down(&self) -> MainMenuSelection {
        self.main_menu_next_enabled(MainMenuSelection::down)
    }

    pub fn main_menu_up(&self) -> MainMenuSelection {
        self.main_menu_next_enabled(MainMenuSelection::up)
    }

    pub fn with_main_menu_selection(&self, selection: MainMenuSelection) -> RunState {
        match self {
            RunState::MainMenu { load_enabled, .. } => RunState::MainMenu {
                selection,
                load_enabled: *load_enabled,
            },
            _ => unimplemented!(),
        }
    }

    // This method is needed only because of
    // #![feature(bindings_after_at)] being unstable
    pub fn main_menu_selection(&self) -> MainMenuSelection {
        match self {
            RunState::MainMenu { selection, .. } => *selection,
            _ => unimplemented!(),
        }
    }
}

impl Default for RunState {
    fn default() -> Self {
        let savegame_exists = LoadSystem::savegame_exists();
        RunState::MainMenu {
            selection: if savegame_exists {
                MainMenuSelection::LoadGame
            } else {
                MainMenuSelection::NewGame
            },
            load_enabled: savegame_exists,
        }
    }
}

macro_attr! {
    #[derive(Clone, PartialEq, Default,
             NewtypeDebug!, NewtypeDeref!, NewtypeDerefMut!, NewtypeFrom!)]
    pub struct RunStateQueue(VecDeque<RunState>);
}
