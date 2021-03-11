//! Cross-platform mouse, keyboard (and gamepads soon) module.

use crate::get_context;
use crate::prelude::screen_height;
use crate::prelude::screen_width;
use crate::Vec2;
pub use miniquad::{KeyCode, MouseButton};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchPhase {
    Started,
    Stationary,
    Moved,
    Ended,
    Cancelled,
}

impl From<miniquad::TouchPhase> for TouchPhase {
    fn from(miniquad_phase: miniquad::TouchPhase) -> TouchPhase {
        match miniquad_phase {
            miniquad::TouchPhase::Started => TouchPhase::Started,
            miniquad::TouchPhase::Moved => TouchPhase::Moved,
            miniquad::TouchPhase::Ended => TouchPhase::Ended,
            miniquad::TouchPhase::Cancelled => TouchPhase::Cancelled,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Vec2,
}

/// Return mouse position in pixels.
pub fn mouse_position() -> (f32, f32) {
    let context = get_context();

    (context.mouse_position.x, context.mouse_position.y)
}

/// Return mouse position in range [-1; 1].
pub fn mouse_position_local() -> Vec2 {
    let (pixels_x, pixels_y) = mouse_position();

    convert_to_local(Vec2::new(pixels_x, pixels_y))
}

/// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
/// If set to false, touches won't affect mouse events.
pub fn is_simulating_mouse_with_touch() -> bool {
    get_context().simulate_mouse_with_touch
}

/// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
/// If set to false, touches won't affect mouse events.
pub fn simulate_mouse_with_touch(option: bool) {
    get_context().simulate_mouse_with_touch = option;
}

/// Return touches with positions in pixels.
pub fn touches() -> Vec<Touch> {
    get_context().touches.values().cloned().collect()
}

/// Return touches with positions in range [-1; 1].
pub fn touches_local() -> Vec<Touch> {
    get_context()
        .touches
        .values()
        .map(|touch| {
            let mut touch = touch.clone();
            touch.position = convert_to_local(touch.position);
            touch
        })
        .collect()
}

pub fn mouse_wheel() -> (f32, f32) {
    let context = get_context();

    (context.mouse_wheel.x, context.mouse_wheel.y)
}

/// Detect if the key has been pressed once
pub fn is_key_pressed(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_pressed.contains(&key_code)
}

/// Detect if the key is being pressed
pub fn is_key_down(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_down.contains(&key_code)
}

/// Return the last pressed char.
/// Each "get_char_pressed" call will consume a character from the input queue.
pub fn get_char_pressed() -> Option<char> {
    let context = get_context();

    context.chars_pressed_queue.pop()
}

/// Return the last pressed key.
pub fn get_last_key_pressed() -> Option<KeyCode> {
    let context = get_context();
    // TODO: this will return a random key from keys_pressed HashMap instead of the last one, fix me later
    context.keys_pressed.iter().next().cloned()
}

/// Detect if the key is being pressed
pub fn is_mouse_button_down(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_down.contains(&btn)
}

/// Detect if the key has been pressed once
pub fn is_mouse_button_pressed(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_pressed.contains(&btn)
}

/// Detect if the key has been released this frame
pub fn is_mouse_button_released(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_released.contains(&btn)
}

/// Convert a position in pixels to a position in the range [-1; 1].
fn convert_to_local(pixel_pos: Vec2) -> Vec2 {
    Vec2::new(pixel_pos.x / screen_width(), pixel_pos.y / screen_height()) * 2.0
        - Vec2::new(1.0, 1.0)
}

/// Repeats all events that came in this frame. This function should be used by external tools that uses miniquad system, like different UI librarires.
pub fn repeat_all_miniquad_input<T: miniquad::EventHandler>(t: &mut T) {
    let context = get_context();
    let mut ctx = &mut context.quad_context;

    for event in &context.input_events {
        use crate::MiniquadInputEvent::*;
        match event {
            MouseMotion { x, y } => t.mouse_motion_event(&mut ctx, *x, *y),
            MouseWheel { x, y } => t.mouse_wheel_event(&mut ctx, *x, *y),
            MouseButtonDown { x, y, btn } => t.mouse_button_down_event(&mut ctx, *btn, *x, *y),
            MouseButtonUp { x, y, btn } => t.mouse_button_up_event(&mut ctx, *btn, *x, *y),
            Char {
                character,
                modifiers,
                repeat,
            } => t.char_event(&mut ctx, *character, *modifiers, *repeat),
            KeyDown {
                keycode,
                modifiers,
                repeat,
            } => t.key_down_event(&mut ctx, *keycode, *modifiers, *repeat),
            KeyUp { keycode, modifiers } => t.key_up_event(&mut ctx, *keycode, *modifiers),
        }
    }
}
