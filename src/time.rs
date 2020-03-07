use crate::get_context;

/// Set target FPS (maximum)
pub fn set_target_fps(_fps: f32) {
    unimplemented!()
}

/// Returns current FPS
pub fn get_fps() -> i32 {
    unimplemented!()
}

/// Returns time in seconds for last frame drawn
pub fn get_frame_time() -> f32 {
    unimplemented!()
}

/// Returns elapsed time in seconds since start
pub fn get_time() -> f64 {
    let context = get_context();

    miniquad::date::now() - context.start_time
}
