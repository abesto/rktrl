use bracket_lib::prelude::*;

/// Copy-paste of bracket_lib::prelude::xp_to_console, updated to work on a DrawBatch
/// instead of directly on a console.
// TODO get rid of this if / when https://github.com/thebracket/bracket-lib/pull/185 is merged and released
pub fn xp_to_draw_batch(xp: &XpFile, draw_batch: &mut DrawBatch, offset_x: i32, offset_y: i32) {
    for layer in &xp.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if !cell.bg.is_transparent() {
                    draw_batch.set(
                        Point::new(x as i32 + offset_x, y as i32 + offset_y),
                        ColorPair::new(RGB::from_xp(cell.fg), RGB::from_xp(cell.bg)),
                        cell.ch as FontCharType,
                    );
                }
            }
        }
    }
}
