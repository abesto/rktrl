use bracket_lib::prelude::{FontCharType, RGB};
use legion::Entity;

use crate::components::Position;
use crate::resources::Input;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Label {
    Root,
    Turn {
        entity: Entity,
    },
    Input {
        input: Input,
    },
    ParticleRequest {
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32,
    },

    // Intents
    SkipBecauseInput,
    SkipBecauseHidden,
    SkipBecauseConfused,
    MoveIntent {
        target: Position,
    },
    MeleeIntent {
        target: Position,
    },

    // Actions (taken)
    MoveAction,
    MeleeAction {
        target: Entity,
    },

    //// Effects
    // Effects - Movement
    MovementDone,
    MovementBlocked,

    // Effects - Combat
    Hit,
    AttackedEmptySpace,
    AttackerIsAlreadyDead,
    TargetIsAlreadyDead,

    // Effects - Damage
    Damage {
        to: Entity,
        amount: i32,
    },
    Death {
        entity: Entity,
    },

    // Effects - Misc
    ConfusionOver {
        entity: Entity,
    },

    // Effects - Hunger
    NoLongerWellFed,
    Hungry,
    Starving,
    HungerPang,
}
