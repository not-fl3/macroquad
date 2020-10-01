use crate::get_context;

/// Set target FPS (maximum)
pub fn set_target_fps(_fps: f32) {
    unimplemented!()
}

/// Returns current FPS
pub fn get_fps() -> i32 {
    let context = get_context();

    (1. / context.frame_time) as i32
}

/// Returns time in seconds for last frame drawn
pub fn get_frame_time() -> f32 {
    let context = get_context();

    context.frame_time as f32
}

/// Returns elapsed time in seconds since start
pub fn get_time() -> f64 {
    let context = get_context();

    miniquad::date::now() - context.start_time
}
