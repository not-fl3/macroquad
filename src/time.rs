//! Cross platform system time access and FPS counters.

use crate::get_context;

/// Struct Clock
pub struct Clock {
    elapsed: f32,
    can_tick: bool
}

impl Clock {
    pub fn new() -> Self {
        Self { elapsed: 0.0, can_tick: true }
    }

    // Restart clock, set elapsed time to 0
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }

    // Update clock
    pub fn tick(&mut self) {
        if self.can_tick {
            self.elapsed += get_frame_time();
        }
    }

    // Stop clock ticking
    pub fn pause(&mut self) {
        self.can_tick = false;
    }

    // Resume clock ticking 
    pub fn resume(&mut self) {
        self.can_tick = true;
    }

    // Get time spent while clock was ticking
    pub fn get_elpased_time(&self) -> f32 {
        self.elapsed
    }

    pub fn on(&mut self, time_spent: f32) -> bool {
        let mut result: bool = false;
        if self.elapsed >= time_spent {
            result = true;
        }
        result
    }
}

/// Returns current FPS
pub fn get_fps() -> i32 {
    let context = get_context();

    (1. / context.frame_time) as i32
}

/// Returns duration in seconds of the last frame drawn
pub fn get_frame_time() -> f32 {
    let context = get_context();

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
pub fn get_time() -> f64 {
    let context = get_context();

    miniquad::date::now() - context.start_time
}
