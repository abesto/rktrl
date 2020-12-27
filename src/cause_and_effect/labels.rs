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
    // Effects
    MovementDone,
    MovementBlocked,
    Hit,
    Damage {
        to: Entity,
        amount: u32,
    },
    Death {
        entity: Entity,
    },
    ConfusionOver {
        entity: Entity,
    },
}
