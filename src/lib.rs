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
//! * Immidiate mode UI library included
//! * Single command deploy for both WASM and Android [build instructions](https://github.com/not-fl3/miniquad/#building-examples)
//! # Example
//! ```
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

use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

mod drawing;
mod exec;

pub mod camera;
pub mod material;
pub mod models;
pub mod shapes;
pub mod texture;
pub mod time;
pub mod input;
pub mod window;
pub mod file;

mod types;
mod ui;

pub mod collections;
pub mod coroutines;

pub mod prelude;

// TODO: write something about macroquad entrypoint
#[doc(hidden)]
pub use macroquad_macro::main;

/// Cross platform random generator.
pub mod rand {
    pub use quad_rand::*;
}

#[cfg(feature = "log-impl")]
/// Logging macroses, available with "log-impl" feature.
pub mod logging {
    pub use miniquad::{debug, error, info, warn};
}

use drawing::DrawContext;
use glam::{vec2, Vec2};
use quad_gl::{colors::*, Color};

struct Context {
    quad_context: QuadContext,

    screen_width: f32,
    screen_height: f32,

    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_wheel: Vec2,

    draw_context: DrawContext,
    coroutines_context: coroutines::CoroutinesContext,

    start_time: f64,
    last_frame_time: f64,
    frame_time: f64,
}

impl Context {
    const DEFAULT_BG_COLOR: Color = BLACK;

    fn new(mut ctx: QuadContext) -> Context {
        let (screen_width, screen_height) = ctx.screen_size();

        Context {
            screen_width,
            screen_height,

            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_position: vec2(0., 0.),
            mouse_wheel: vec2(0., 0.),

            draw_context: DrawContext::new(&mut ctx),

            quad_context: ctx,
            coroutines_context: coroutines::CoroutinesContext::new(),

            start_time: miniquad::date::now(),
            last_frame_time: miniquad::date::now(),
            frame_time: 1. / 60.,
        }
    }

    fn begin_frame(&mut self) {
        self.clear(Self::DEFAULT_BG_COLOR);
        self.draw_context
            .update_projection_matrix(&mut self.quad_context);
    }

    fn end_frame(&mut self) {
        self.draw_context
            .perform_render_passes(&mut self.quad_context);

        self.draw_context.ui.new_frame();

        self.quad_context.commit_frame();

        self.mouse_wheel = Vec2::new(0., 0.);
        self.keys_pressed.clear();
    }

    fn clear(&mut self, color: Color) {
        self.quad_context.clear(
            Some((
                color.0[0] as f32 / 255.0,
                color.0[1] as f32 / 255.0,
                color.0[2] as f32 / 255.0,
                color.0[3] as f32 / 255.0,
            )),
            None,
            None,
        );
        self.draw_context.gl.reset();
        self.draw_context
            .update_projection_matrix(&mut self.quad_context);
    }
}

static mut CONTEXT: Option<Context> = None;

fn get_context() -> &'static mut Context {
    unsafe { CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

static mut MAIN_FUTURE: Option<Pin<Box<dyn Future<Output = ()>>>> = None;

struct Stage {}

impl EventHandlerFree for Stage {
    fn resize_event(&mut self, width: f32, height: f32) {
        get_context().screen_width = width;
        get_context().screen_height = height;
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        use megaui::InputHandler;

        let context = get_context();

        context.mouse_position = Vec2::new(x, y);
        context.draw_context.ui.mouse_move((x, y));
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        use megaui::InputHandler;

        let context = get_context();

        context.mouse_wheel.set_x(x);
        context.mouse_wheel.set_y(y);

        context.draw_context.ui.mouse_wheel(x, -y);
    }
    fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        use megaui::InputHandler;

        let context = get_context();

        context.mouse_pressed.insert(btn);
        context.draw_context.ui.mouse_down((x, y));
    }

    fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        use megaui::InputHandler;

        let context = get_context();

        context.mouse_pressed.remove(&btn);

        context.draw_context.ui.mouse_up((x, y));
    }

    fn char_event(&mut self, character: char, modifiers: KeyMods, _repeat: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context
            .draw_context
            .ui
            .char_event(character, modifiers.shift, modifiers.ctrl);
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context.keys_down.insert(keycode);
        if repeat == false {
            context.keys_pressed.insert(keycode);
        }

        match keycode {
            KeyCode::Up => context.draw_context.ui.key_down(
                megaui::KeyCode::Up,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Down => context.draw_context.ui.key_down(
                megaui::KeyCode::Down,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Right => context.draw_context.ui.key_down(
                megaui::KeyCode::Right,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Left => context.draw_context.ui.key_down(
                megaui::KeyCode::Left,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Home => context.draw_context.ui.key_down(
                megaui::KeyCode::Home,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::End => context.draw_context.ui.key_down(
                megaui::KeyCode::End,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Delete => context.draw_context.ui.key_down(
                megaui::KeyCode::Delete,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Backspace => context.draw_context.ui.key_down(
                megaui::KeyCode::Backspace,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Enter => context.draw_context.ui.key_down(
                megaui::KeyCode::Enter,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Tab => context.draw_context.ui.key_down(
                megaui::KeyCode::Tab,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Z => context.draw_context.ui.key_down(
                megaui::KeyCode::Z,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::Y => context.draw_context.ui.key_down(
                megaui::KeyCode::Y,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::C => context.draw_context.ui.key_down(
                megaui::KeyCode::C,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::X => context.draw_context.ui.key_down(
                megaui::KeyCode::X,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::V => context.draw_context.ui.key_down(
                megaui::KeyCode::V,
                modifiers.shift,
                modifiers.ctrl,
            ),
            KeyCode::A => context.draw_context.ui.key_down(
                megaui::KeyCode::A,
                modifiers.shift,
                modifiers.ctrl,
            ),
            _ => {}
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        let context = get_context();
        context.keys_down.remove(&keycode);
    }

    fn update(&mut self) {}

    fn draw(&mut self) {
        if let Some(future) = unsafe { MAIN_FUTURE.as_mut() } {
            get_context().begin_frame();

            if exec::resume(future) {
                unsafe {
                    MAIN_FUTURE = None;
                }
                get_context().quad_context.quit();
                return;
            }
            get_context().coroutines_context.update();
        }

        get_context().end_frame();

        get_context().frame_time = date::now() - get_context().last_frame_time;
        get_context().last_frame_time = date::now();
    }
}

/// Not meant to be used directly, only from the macro.
#[doc(hidden)]
pub struct Window {}

impl Window {
    pub fn new(label: &str, future: impl Future<Output = ()> + 'static) {
        Window::from_config(
            conf::Conf {
                sample_count: 4,
                window_title: label.to_string(),
                ..Default::default()
            },
            future,
        );
    }

    pub fn from_config(config: conf::Conf, future: impl Future<Output = ()> + 'static) {
        miniquad::start(
            conf::Conf {
                sample_count: 4,
                ..config
            },
            |ctx| {
                unsafe {
                    MAIN_FUTURE = Some(Box::pin(future));
                }
                unsafe { CONTEXT = Some(Context::new(ctx)) };
                UserData::free(Stage {})
            },
        );
    }
}
