use crate::components::Position;
use crate::resources::Input;
use legion::Entity;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Label {
    Root,
    Turn { entity: Entity },
    Input { input: Input },
    // Intents
    SkipBecauseHidden,
    SkipBecauseConfused,
    MoveIntent { target: Position },
    MeleeIntent { target: Position },
    // Actions (taken)
    MoveAction,
    MeleeAction { target: Entity },
    // Effects
    MovementDone,
    MovementBlocked,
    Hit,
    Damage { to: Entity, amount: u32 },
    Death { entity: Entity },
    ConfusionOver { entity: Entity },
}
