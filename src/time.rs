//! Cross platform system time access and FPS counters.

/// Returns current FPS
pub fn get_fps(context: &crate::Context) -> i32 {
    (1. / context.frame_time) as i32
}

/// Returns duration in seconds of the last frame drawn
pub fn get_frame_time(context: &crate::Context) -> f32 {
    if crate::experimental::scene::in_fixed_update() {
        crate::experimental::scene::fixed_frame_time()
    } else {
        context.frame_time as f32
    }
}

/// Returns elapsed wall-clock time in seconds since start
///
/// Note that as real world time progresses during computation,
/// the value returned will change. Therefore if you want
/// your game logic update to happen at the same *in-game* time
/// for all game objects, you should call this function once
/// save the value and reuse it throughout your code.
pub fn get_time(context: &crate::Context) -> f64 {
    miniquad::date::now() - context.start_time
}
