//! Cross-platform mouse, keyboard (and gamepads soon) module.

use crate::{vec2, Context, Vec2};
pub use miniquad::{KeyCode, MouseButton};

use std::collections::{HashMap, HashSet};

pub(crate) struct InputContext {
    pub simulate_mouse_with_touch: bool,
    pub keys_down: HashSet<KeyCode>,
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_released: HashSet<KeyCode>,
    pub mouse_down: HashSet<MouseButton>,
    pub mouse_pressed: HashSet<MouseButton>,
    pub mouse_released: HashSet<MouseButton>,
    pub touches: HashMap<u64, Touch>,
    pub chars_pressed_queue: Vec<char>,
    pub chars_pressed_ui_queue: Vec<char>,
    pub mouse_position: Vec2,
    pub last_mouse_position: Option<Vec2>,
    pub mouse_raw_delta: Vec2,
    pub mouse_wheel: Vec2,
}

impl InputContext {
    pub fn new() -> InputContext {
        InputContext {
            simulate_mouse_with_touch: true,

            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            chars_pressed_queue: Vec::new(),
            chars_pressed_ui_queue: Vec::new(),
            mouse_down: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
            touches: HashMap::new(),
            mouse_position: vec2(0., 0.),
            last_mouse_position: None,
            mouse_raw_delta: vec2(0., 0.),
            mouse_wheel: vec2(0., 0.),
        }
    }

    pub fn end_frame(&mut self) {
        self.mouse_wheel = Vec2::new(0., 0.);
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();

        //self.last_mouse_position = Some(self.mouse_position_local());

        // remove all touches that were Ended or Cancelled
        self.touches.retain(|_, touch| {
            touch.phase != TouchPhase::Ended && touch.phase != TouchPhase::Cancelled
        });

        // change all Started or Moved touches to Stationary
        for touch in self.touches.values_mut() {
            if touch.phase == TouchPhase::Started || touch.phase == TouchPhase::Moved {
                touch.phase = TouchPhase::Stationary;
            }
        }
    }
}
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

// /// Constrain mouse to window
// pub fn set_cursor_grab(grab: bool) {
//     let context = get_context();
//     context.cursor_grabbed = grab;
//     miniquad::window::set_cursor_grab(grab);
// }

// /// Set mouse cursor visibility
// pub fn show_mouse(shown: bool) {
//     miniquad::window::show_mouse(shown);
// }

// pub fn mouse_raw_delta_position() -> Vec2 {
//     let context = get_context();

//     context.mouse_raw_delta
// }

// /// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
// /// If set to false, touches won't affect mouse events.
// pub fn is_simulating_mouse_with_touch() -> bool {
//     get_context().simulate_mouse_with_touch
// }

// /// This is set to true by default, meaning touches will raise mouse events in addition to raising touch events.
// /// If set to false, touches won't affect mouse events.
// pub fn simulate_mouse_with_touch(option: bool) {
//     get_context().simulate_mouse_with_touch = option;
// }

// /// Return touches with positions in pixels.
// pub fn touches() -> Vec<Touch> {
//     get_context().touches.values().cloned().collect()
// }

// /// Return touches with positions in range [-1; 1].
// pub fn touches_local() -> Vec<Touch> {
//     get_context()
//         .touches
//         .values()
//         .map(|touch| {
//             let mut touch = touch.clone();
//             touch.position = convert_to_local(touch.position);
//             touch
//         })
//         .collect()
// }

// /// Detect if the key has been pressed once
// pub fn is_key_pressed(key_code: KeyCode) -> bool {
//     let context = get_context();

//     context.keys_pressed.contains(&key_code)
// }

// /// Detect if the key is being pressed
// pub fn is_key_down(key_code: KeyCode) -> bool {
//     let context = get_context();

//     context.keys_down.contains(&key_code)
// }

// /// Detect if the key has been released this frame
// pub fn is_key_released(key_code: KeyCode) -> bool {
//     let context = get_context();

//     context.keys_released.contains(&key_code)
// }

// /// Return the last pressed char.
// /// Each "get_char_pressed" call will consume a character from the input queue.
// pub fn get_char_pressed() -> Option<char> {
//     let context = get_context();

//     context.chars_pressed_queue.pop()
// }

// pub(crate) fn get_char_pressed_ui() -> Option<char> {
//     let context = get_context();

//     context.chars_pressed_ui_queue.pop()
// }

// /// Return the last pressed key.
// pub fn get_last_key_pressed() -> Option<KeyCode> {
//     let context = get_context();
//     // TODO: this will return a random key from keys_pressed HashMap instead of the last one, fix me later
//     context.keys_pressed.iter().next().cloned()
// }

impl Context {
    /// Detect if the button is being pressed
    pub fn is_mouse_button_down(&self, btn: MouseButton) -> bool {
        let context = self.input.lock().unwrap();

        context.mouse_down.contains(&btn)
    }

    // /// Returns the difference between the current mouse position and the mouse position on the previous frame.
    // pub fn mouse_delta(&self) -> Vec2 {
    //     let current_position = self.mouse_position_local();
    //     let context = self.input.lock().unwrap();
    //     let last_position = context.last_mouse_position.unwrap_or(current_position);

    //     // Calculate the delta
    //     let delta = last_position - current_position;

    //     delta
    // }

    pub fn mouse_wheel(&self) -> (f32, f32) {
        let context = self.input.lock().unwrap();

        (context.mouse_wheel.x, context.mouse_wheel.y)
    }

    /// Return mouse position in pixels.
    pub fn mouse_position(&self) -> Vec2 {
        let context = self.input.lock().unwrap();

        vec2(
            context.mouse_position.x / miniquad::window::dpi_scale(),
            context.mouse_position.y / miniquad::window::dpi_scale(),
        )
    }

    // /// Return mouse position in range [-1; 1].
    // pub fn mouse_position_local(&self) -> Vec2 {
    //     let m = self.mouse_position();

    //     self.convert_to_local(m)
    // }

    // /// Convert a position in pixels to a position in the range [-1; 1].
    // fn convert_to_local(&self, pixel_pos: Vec2) -> Vec2 {
    //     let context = self.input.lock().unwrap();

    //     Vec2::new(pixel_pos.x / screen_width(), pixel_pos.y / screen_height()) * 2.0
    //         - Vec2::new(1.0, 1.0)
    // }
}

impl InputContext {
    /// Return mouse position in pixels.
    pub fn mouse_position(&self) -> Vec2 {
        vec2(
            self.mouse_position.x / miniquad::window::dpi_scale(),
            self.mouse_position.y / miniquad::window::dpi_scale(),
        )
    }

    // /// Return mouse position in range [-1; 1].
    // pub fn mouse_position_local(&self) -> Vec2 {
    //     let m = self.mouse_position();

    //     self.convert_to_local(m)
    // }

    // /// Convert a position in pixels to a position in the range [-1; 1].
    // fn convert_to_local(&self, pixel_pos: Vec2) -> Vec2 {
    //     Vec2::new(pixel_pos.x / screen_width(), pixel_pos.y / screen_height()) * 2.0
    //         - Vec2::new(1.0, 1.0)
    // }
}
// /// Detect if the button has been pressed once
// pub fn is_mouse_button_pressed(btn: MouseButton) -> bool {
//     let context = get_context();

//     context.mouse_pressed.contains(&btn)
// }

// /// Detect if the button has been released this frame
// pub fn is_mouse_button_released(btn: MouseButton) -> bool {
//     let context = get_context();

//     context.mouse_released.contains(&btn)
// }

// /// Prevents quit
// pub fn prevent_quit() {
//     get_context().prevent_quit_event = true;
// }

// /// Detect if quit has been requested
// pub fn is_quit_requested() -> bool {
//     get_context().quit_requested
// }

// /// Functions for advanced input processing.
// ///
// /// Functions in this module should be used by external tools that uses miniquad system, like different UI libraries. User shouldn't use this function.
// pub mod utils {
//     use crate::get_context;

//     /// Register input subscriber. Returns subscriber identifier that must be used in `repeat_all_miniquad_input`.
//     pub fn register_input_subscriber() -> usize {
//         let context = get_context();

//         context.input_events.push(vec![]);

//         context.input_events.len() - 1
//     }

//     /// Repeats all events that came since last call of this function with current value of `subscriber`. This function must be called at each frame.
//     pub fn repeat_all_miniquad_input<T: miniquad::EventHandler>(t: &mut T, subscriber: usize) {
//         let context = get_context();

//         for event in &context.input_events[subscriber] {
//             event.repeat(t);
//         }
//         context.input_events[subscriber].clear();
//     }
// }
