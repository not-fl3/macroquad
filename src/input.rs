//! Cross-platform mouse, keyboard (and gamepads soon) module.

use crate::prelude::screen_width;
use crate::prelude::screen_height;
use crate::Vec2;
use crate::get_context;
pub use miniquad::{KeyCode, MouseButton};

/// Return mouse position in pixels.
pub fn mouse_position() -> (f32, f32) {
    let context = get_context();

    (context.mouse_position.x, context.mouse_position.y)
}

/// Return mouse position in range [-1; 1].
pub fn mouse_position_local() -> Vec2 {
    let (pixels_x, pixels_y) = mouse_position();

    Vec2::new(pixels_x / screen_width(), pixels_y / screen_height()) * 2.0 - Vec2::new(1.0, 1.0)
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
