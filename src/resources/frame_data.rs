use bracket_lib::prelude::*;

#[derive(Debug)]
pub struct FrameData {
    pub fps: f32,
    pub frame_time_ms: f32,
}

impl From<&BTerm> for FrameData {
    fn from(bterm: &BTerm) -> Self {
        FrameData {
            fps: bterm.fps,
            frame_time_ms: bterm.frame_time_ms,
        }
    }
}
