use bracket_lib::prelude::{embedded_resource, link_resource, XpFile, EMBED};

macro_rules! asset_path {
    (SMALL_DUNGEON) => {
        "../../assets/SmallDungeon_80x50.xp"
    };
}

embedded_resource!(SMALL_DUNGEON, asset_path!(SMALL_DUNGEON));

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        link_resource!(SMALL_DUNGEON, asset_path!(SMALL_DUNGEON));

        RexAssets {
            menu: XpFile::from_resource(asset_path!(SMALL_DUNGEON)).unwrap(),
        }
    }
}

impl Default for RexAssets {
    fn default() -> Self {
        RexAssets::new()
    }
}
