//!
//! `macroquad` is a simple and easy to use game library for Rust programming language.
//!
//! `macroquad` attempts to avoid any rust-specific programming concepts like lifetimes/borrowing, making it very friendly for rust beginners.
//!
//! ## Supported platforms
//!
//! * PC: Windows/Linux/MacOS
//! * HTML5
//! * Android
//! * IOS
//!
//! ## Features
//!
//! * Same code for all supported platforms, no platform dependent defines required
//! * Efficient 2D rendering with automatic geometry batching
//! * Minimal amount of dependencies: build after `cargo clean` takes only 16s on x230(~6years old laptop)
//! * Immediate mode UI library included
//! * Single command deploy for both WASM and Android [build instructions](https://github.com/not-fl3/miniquad/#building-examples)
//! # Example
//! ```no_run
//! use macroquad::prelude::*;
//!
//! #[macroquad::main("BasicShapes")]
//! async fn main() {
//!     loop {
//!         clear_background(RED);
//!
//!         draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
//!         draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
//!         draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
//!         draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);
//!
//!         next_frame().await
//!     }
//! }
//!```
#![allow(warnings)]
use miniquad::*;

use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

mod exec;

pub mod file;
pub mod input;
pub mod time;
pub mod window;

pub mod telemetry;

mod error;

pub use error::Error;

/// Cross platform random generator.
pub mod rand {
    pub use quad_rand::*;
}

#[cfg(not(feature = "log-rs"))]
/// Logging macros, available with miniquad "log-impl" feature.
pub mod logging {
    pub use miniquad::{debug, error, info, trace, warn};
}
#[cfg(feature = "log-rs")]
// Use logging facade
pub use ::log as logging;
pub use miniquad;

pub mod compat;

use glam::{vec2, Mat4, Vec2};
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone)]
enum MiniquadInputEvent {
    MouseMotion {
        x: f32,
        y: f32,
    },
    MouseWheel {
        x: f32,
        y: f32,
    },
    MouseButtonDown {
        x: f32,
        y: f32,
        btn: MouseButton,
    },
    MouseButtonUp {
        x: f32,
        y: f32,
        btn: MouseButton,
    },
    Char {
        character: char,
        modifiers: KeyMods,
        repeat: bool,
    },
    KeyDown {
        keycode: KeyCode,
        modifiers: KeyMods,
        repeat: bool,
    },
    KeyUp {
        keycode: KeyCode,
        modifiers: KeyMods,
    },
    Touch {
        phase: TouchPhase,
        id: u64,
        x: f32,
        y: f32,
    },
}

impl MiniquadInputEvent {
    fn repeat<T: miniquad::EventHandler>(&self, t: &mut T) {
        use crate::MiniquadInputEvent::*;
        match self {
            MouseMotion { x, y } => t.mouse_motion_event(*x, *y),
            MouseWheel { x, y } => t.mouse_wheel_event(*x, *y),
            MouseButtonDown { x, y, btn } => t.mouse_button_down_event(*btn, *x, *y),
            MouseButtonUp { x, y, btn } => t.mouse_button_up_event(*btn, *x, *y),
            Char {
                character,
                modifiers,
                repeat,
            } => t.char_event(*character, *modifiers, *repeat),
            KeyDown {
                keycode,
                modifiers,
                repeat,
            } => t.key_down_event(*keycode, *modifiers, *repeat),
            KeyUp { keycode, modifiers } => t.key_up_event(*keycode, *modifiers),
            Touch { phase, id, x, y } => t.touch_event(*phase, *id, *x, *y),
        }
    }
}

// This is required for #[macroquad::test]
//
// unfortunately #[cfg(test)] do not work with integration tests
// so this module should be publicly available
#[doc(hidden)]
pub mod test {
    pub static mut MUTEX: Option<std::sync::Mutex<()>> = None;
    pub static ONCE: std::sync::Once = std::sync::Once::new();
}

struct Stage {
    main_future: Option<Pin<Box<dyn Future<Output = ()>>>>,
    ctx: Arc<Context>,
}

impl EventHandler for Stage {
    fn resize_event(&mut self, width: f32, height: f32) {
        let _z = telemetry::ZoneGuard::new("Event::resize_event");
        // get_context().screen_width = width;
        // get_context().screen_height = height;
    }

    // fn raw_mouse_motion(&mut self, x: f32, y: f32) {
    //     let context = get_context();

    //     context.mouse_raw_delta = vec2(x, y);
    //     // if context.cursor_grabbed {
    //     //     //context.mouse_position += Vec2::new(x, y);

    //     //     let event = MiniquadInputEvent::MouseMotion {
    //     //         x: context.mouse_position.x,
    //     //         y: context.mouse_position.y,
    //     //     };
    //     //     context
    //     //         .input_events
    //     //         .iter_mut()
    //     //         .for_each(|arr| arr.push(event.clone()));
    //     // }
    // }

    fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        let mut context = self.ctx.input.lock().unwrap();

        context.mouse_down.insert(btn);
        context.mouse_pressed.insert(btn);

        // context
        //     .input_events
        //     .iter_mut()
        //     .for_each(|arr| arr.push(MiniquadInputEvent::MouseButtonDown { x, y, btn }));

        // if !context.cursor_grabbed {
        //     context.mouse_position = Vec2::new(x, y);
        // }
    }

    fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        let mut context = self.ctx.input.lock().unwrap();

        context.mouse_down.remove(&btn);
        context.mouse_released.insert(btn);

        // context
        //     .input_events
        //     .iter_mut()
        //     .for_each(|arr| arr.push(MiniquadInputEvent::MouseButtonUp { x, y, btn }));

        // if !context.cursor_grabbed {
        //     context.mouse_position = Vec2::new(x, y);
        // }
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let mut context = self.ctx.input.lock().unwrap();

        //if !context.cursor_grabbed {
        context.mouse_position = Vec2::new(x, y);
        //}
        //     context
        //         .input_events
        //         .iter_mut()
        //         .for_each(|arr| arr.push(MiniquadInputEvent::MouseMotion { x, y }));
        // }
    }

    // fn mouse_wheel_event(&mut self, x: f32, y: f32) {
    //     let context = get_context();

    //     context.mouse_wheel.x = x;
    //     context.mouse_wheel.y = y;

    //     context
    //         .input_events
    //         .iter_mut()
    //         .for_each(|arr| arr.push(MiniquadInputEvent::MouseWheel { x, y }));
    // }

    // fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
    //     let context = get_context();

    //     context.touches.insert(
    //         id,
    //         input::Touch {
    //             id,
    //             phase: phase.into(),
    //             position: Vec2::new(x, y),
    //         },
    //     );

    //     if context.simulate_mouse_with_touch {
    //         if phase == TouchPhase::Started {
    //             self.mouse_button_down_event(MouseButton::Left, x, y);
    //         }

    //         if phase == TouchPhase::Ended {
    //             self.mouse_button_up_event(MouseButton::Left, x, y);
    //         }

    //         if phase == TouchPhase::Moved {
    //             self.mouse_motion_event(x, y);
    //         }
    //     };

    //     context
    //         .input_events
    //         .iter_mut()
    //         .for_each(|arr| arr.push(MiniquadInputEvent::Touch { phase, id, x, y }));
    // }

    // fn char_event(&mut self, character: char, modifiers: KeyMods, repeat: bool) {
    //     let context = get_context();

    //     context.chars_pressed_queue.push(character);
    //     context.chars_pressed_ui_queue.push(character);

    //     context.input_events.iter_mut().for_each(|arr| {
    //         arr.push(MiniquadInputEvent::Char {
    //             character,
    //             modifiers,
    //             repeat,
    //         })
    //     });
    // }

    // fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
    //     let context = get_context();
    //     context.keys_down.insert(keycode);
    //     if repeat == false {
    //         context.keys_pressed.insert(keycode);
    //     }

    //     context.input_events.iter_mut().for_each(|arr| {
    //         arr.push(MiniquadInputEvent::KeyDown {
    //             keycode,
    //             modifiers,
    //             repeat,
    //         })
    //     });
    // }

    // fn key_up_event(&mut self, keycode: KeyCode, modifiers: KeyMods) {
    //     let context = get_context();
    //     context.keys_down.remove(&keycode);
    //     context.keys_released.insert(keycode);

    //     context
    //         .input_events
    //         .iter_mut()
    //         .for_each(|arr| arr.push(MiniquadInputEvent::KeyUp { keycode, modifiers }));
    // }

    fn update(&mut self) {
        let _z = telemetry::ZoneGuard::new("Event::update");

        // Unless called every frame, cursor will not remain grabbed
        //miniquad::window::set_cursor_grab(get_context().cursor_grabbed);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // TODO: consider making it a part of miniquad?
            std::thread::yield_now();
        }
    }

    fn draw(&mut self) {
        //let result = maybe_unwind(get_context().unwind, || {
        if let Some(future) = self.main_future.as_mut() {
            let _z = telemetry::ZoneGuard::new("user code");

            if exec::resume(future).is_some() {
                self.main_future = None;
                miniquad::window::quit();
                return;
            }

            compat::end_frame();
            //get_context().coroutines_context.update();
        }
    }
    
    fn window_restored_event(&mut self) {
        #[cfg(target_os = "android")]
        get_context().audio_context.resume();
    }

    fn window_minimized_event(&mut self) {
        #[cfg(target_os = "android")]
        get_context().audio_context.pause();
    }

    fn quit_requested_event(&mut self) {
        // let context = get_context();
        // if context.prevent_quit_event {
        //     miniquad::window::cancel_quit();
        //     context.quit_requested = true;
        // }
    }
}

#[derive(Clone)]
pub struct Context {
    pub quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    pub quad_gl: quad_gl::QuadGl,
    pub(crate) input: Arc<Mutex<input::InputContext>>,
}

impl Context {
    pub(crate) fn new() -> Context {
        let quad_ctx = miniquad::window::new_rendering_backend();
        let quad_ctx = Arc::new(Mutex::new(quad_ctx));
        Context {
            quad_ctx: quad_ctx.clone(),
            quad_gl: quad_gl::QuadGl::new(quad_ctx),
            input: Arc::new(Mutex::new(input::InputContext::new())),
        }
    }

    // pub fn new_scene(&self) -> scene::Scene {
    //     scene::Scene::new(self.quad_ctx.clone(), self.fonts_storage.clone())
    // }

    // pub fn new_canvas(&self) -> sprite_layer::SpriteLayer {
    //     //sprite_layer::SpriteLayer::new(self.quad_ctx.clone(), self.fonts_storage.clone())
    // }
}

pub fn start<F: Fn(Context) -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
    mut config: conf::Conf,
    future: F,
) {
    miniquad::start(conf::Conf { ..config }, move || {
        let ctx = Context::new();
        Box::new(Stage {
            main_future: Some(Box::pin(future(ctx.clone()))),
            ctx: Arc::new(ctx),
        })
    });
}
