/// Clean up entities when moving to the next level
use specs::prelude::*;

use crate::components::player::Player;

use rktrl_macros::systemdata;

systemdata!(NextLevelCleanupSystemData(entities read(Player)));

//impl System<'a> for NextLevelCleanupSystem<'a> {}
