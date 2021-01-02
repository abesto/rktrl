use bracket_lib::prelude::{FontCharType, RGB};
use legion::Entity;

use crate::systems::prelude::{Input, Position};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UseTarget {
    SelfCast,
    Position(Position),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Label {
    Root,
    Turn {
        actor: Entity,
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
        target_position: Position,
    },
    NextLevelIntent,
    MeleeIntent {
        target_position: Position,
    },
    PickupIntent,
    DropIntent {
        item: Entity,
    },
    RemoveIntent {
        item: Entity,
    },
    UseIntent {
        item: Entity,
        target: UseTarget,
    },

    // Actions (when some data translation is needed from the intent)
    MeleeAction {
        target: Entity,
    },
    PickupAction {
        item: Entity,
    },
    UseOnTarget {
        item: Entity,
        target: Entity,
    },

    //// Effects
    // Effects - Movement
    MovementDone,
    MovementBlocked,
    NoStairsHere,
    MovedToNextLevel,

    // Effects - Combat
    Hit,
    AttackedEmptySpace,
    AttackerIsAlreadyDead,
    TargetIsAlreadyDead,

    // Effects - Health
    Damage {
        to: Entity,
        amount: i32,
        bleeding: bool,
    },
    Death {
        entity: Entity,
    },
    Healing {
        to: Entity,
        amount: i32,
    },

    // Effects - Misc
    Confused {
        entity: Entity,
    },
    ConfusionOver {
        entity: Entity,
    },
    EntryTriggered {
        trigger: Entity,
    },
    Spotted {
        hidden: Entity,
    },

    // Effects - Pickup
    PickupNothingHere,
    PickupDone,

    // Effects - Inventory management
    EquipDone,
    DropDone,
    RemoveDone,

    // Effects - Hunger
    Ate {
        who: Entity,
        what: Entity,
    },
    NoLongerWellFed,
    Hungry,
    Starving,
    HungerPang,

    // Effects - Item use
    TooFarAway,
    NoValidTargets,
    MagicMapping,
}
