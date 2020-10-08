//! Cross-platform mouse, keyboard (and gamepads soon) module. 

pub use miniquad::{KeyCode, MouseButton};
use crate::get_context;

pub fn mouse_position() -> (f32, f32) {
    let context = get_context();

    (context.mouse_position.x(), context.mouse_position.y())
}

pub fn mouse_wheel() -> (f32, f32) {
    let context = get_context();

    (context.mouse_wheel.x(), context.mouse_wheel.y())
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

/// Detect if the key is being pressed.
/// Each call "is_key_down" call will consume a character from input queue.
pub fn get_key_pressed() -> Option<char> {
    let context = get_context();

    context.chars_pressed_queue.pop()
}

pub fn is_mouse_button_down(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_pressed.contains(&btn)
}

/// Check for megaui mouse overlap
pub fn mouse_over_ui() -> bool {
    let context = get_context();

    context.draw_context.ui.is_mouse_over(megaui::Vector2::new(
        context.mouse_position.x(),
        context.mouse_position.y(),
    ))
}
