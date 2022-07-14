//! Cross platform system time access and FPS counters.

use crate::get_context;

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

#[derive(PartialEq)]
pub enum TimerState {
    Running,
    Paused,
    Finished,
}

pub struct Timer {
    duration: f32,
    elapsed: f32,
    state: TimerState,
}

impl Timer {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            state: TimerState::Running,
        }
    }

    pub fn tick(&mut self) {
        if self.state != TimerState::Running {
            return;
        }

        self.elapsed += get_frame_time();

        if self.elapsed > self.duration {
            self.finish();
        }
    }

    pub fn pause(&mut self) {
        self.state = TimerState::Paused;
    }

    pub fn finish(&mut self) {
        self.state = TimerState::Finished;
    }

    pub fn start(&mut self) {
        self.state = TimerState::Running;
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn get_state(&self) -> &TimerState {
        return &self.state;
    }

    pub fn get_elapsed(&self) -> f32 {
        return self.elapsed;
    }
}