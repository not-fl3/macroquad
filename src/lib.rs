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

use miniquad::Context as QuadContext;
use miniquad::*;

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

mod drawing;
mod exec;
mod quad_gl;

pub mod camera;
pub mod color;
pub mod file;
pub mod input;
pub mod material;
pub mod math;
pub mod models;
pub mod shapes;
pub mod text;
pub mod texture;
pub mod time;
pub mod ui;
pub mod window;

pub mod experimental;

pub mod prelude;

pub mod telemetry;

// TODO: write something about macroquad entrypoint
#[doc(hidden)]
pub use macroquad_macro::main;

/// Cross platform random generator.
pub mod rand {
    pub use quad_rand::*;
}

#[cfg(feature = "log-impl")]
/// Logging macros, available with "log-impl" feature.
pub mod logging {
    pub use miniquad::{debug, error, info, warn};
}
pub use miniquad;

use drawing::DrawContext;
use glam::{vec2, Vec2};
use quad_gl::{colors::*, Color};
use ui::ui_context::UiContext;

struct Context {
    quad_context: QuadContext,

    screen_width: f32,
    screen_height: f32,

    simulate_mouse_with_touch: bool,

    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    mouse_down: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    touches: HashMap<u64, input::Touch>,
    chars_pressed_queue: Vec<char>,
    mouse_position: Vec2,
    mouse_wheel: Vec2,

    input_events: Vec<MiniquadInputEvent>,

    draw_context: DrawContext,
    ui_context: UiContext,
    coroutines_context: experimental::coroutines::CoroutinesContext,
    fonts_storage: text::FontsStorage,

    start_time: f64,
    last_frame_time: f64,
    frame_time: f64,
}

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
}

impl Context {
    const DEFAULT_BG_COLOR: Color = BLACK;

    fn new(mut ctx: QuadContext) -> Context {
        let (screen_width, screen_height) = ctx.screen_size();

        Context {
            screen_width,
            screen_height,

            simulate_mouse_with_touch: true,

            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            chars_pressed_queue: Vec::new(),
            mouse_down: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
            touches: HashMap::new(),
            mouse_position: vec2(0., 0.),
            mouse_wheel: vec2(0., 0.),

            input_events: Vec::new(),

            draw_context: DrawContext::new(&mut ctx),
            ui_context: UiContext::new(&mut ctx),
            fonts_storage: text::FontsStorage::new(&mut ctx),

            quad_context: ctx,
            coroutines_context: experimental::coroutines::CoroutinesContext::new(),

            start_time: miniquad::date::now(),
            last_frame_time: miniquad::date::now(),
            frame_time: 1. / 60.,
        }
    }

    fn begin_frame(&mut self) {
        telemetry::begin_gpu_query("GPU");

        self.ui_context.process_input();
        self.clear(Self::DEFAULT_BG_COLOR);
        self.draw_context
            .update_projection_matrix(&mut self.quad_context);
    }

    fn end_frame(&mut self) {
        self.ui_context.draw();

        self.draw_context
            .perform_render_passes(&mut self.quad_context);

        self.quad_context.commit_frame();

        telemetry::end_gpu_query();

        self.mouse_wheel = Vec2::new(0., 0.);
        self.keys_pressed.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();

        // remove all touches that were Ended or Cancelled
        self.touches.retain(|_, touch| {
            touch.phase != input::TouchPhase::Ended && touch.phase != input::TouchPhase::Cancelled
        });

        // change all Started or Moved touches to Stationary
        for touch in self.touches.values_mut() {
            if touch.phase == input::TouchPhase::Started || touch.phase == input::TouchPhase::Moved
            {
                touch.phase = input::TouchPhase::Stationary;
            }
        }
    }

    fn clear(&mut self, color: Color) {
        self.quad_context
            .clear(Some((color.r, color.g, color.b, color.a)), None, None);
        self.draw_context.gl.reset();
        self.draw_context
            .update_projection_matrix(&mut self.quad_context);
    }
}

#[no_mangle]
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
        let context = get_context();

        context.mouse_position = Vec2::new(x, y);

        context
            .input_events
            .push(MiniquadInputEvent::MouseMotion { x, y });
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        let context = get_context();

        context.mouse_wheel.x = x;
        context.mouse_wheel.y = y;

        context
            .input_events
            .push(MiniquadInputEvent::MouseWheel { x, y });
    }
    fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        let context = get_context();

        context.mouse_position = Vec2::new(x, y);
        context.mouse_down.insert(btn);
        context.mouse_pressed.insert(btn);

        context
            .input_events
            .push(MiniquadInputEvent::MouseButtonDown { x, y, btn });
    }

    fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        let context = get_context();

        context.mouse_position = Vec2::new(x, y);
        context.mouse_down.remove(&btn);
        context.mouse_released.insert(btn);

        context
            .input_events
            .push(MiniquadInputEvent::MouseButtonUp { x, y, btn });
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        let context = get_context();

        context.touches.insert(
            id,
            input::Touch {
                id,
                phase: phase.into(),
                position: Vec2::new(x, y),
            },
        );

        if context.simulate_mouse_with_touch {
            if phase == TouchPhase::Started {
                self.mouse_button_down_event(MouseButton::Left, x, y);
            }

            if phase == TouchPhase::Ended {
                self.mouse_button_up_event(MouseButton::Left, x, y);
            }

            if phase == TouchPhase::Moved {
                self.mouse_motion_event(x, y);
            }
        };
    }

    fn char_event(&mut self, character: char, modifiers: KeyMods, repeat: bool) {
        let context = get_context();

        context.chars_pressed_queue.push(character);

        context.input_events.push(MiniquadInputEvent::Char {
            character,
            modifiers,
            repeat,
        });
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
        let context = get_context();
        context.keys_down.insert(keycode);
        if repeat == false {
            context.keys_pressed.insert(keycode);
        }

        context.input_events.push(MiniquadInputEvent::KeyDown {
            keycode,
            modifiers,
            repeat,
        });
    }

    fn key_up_event(&mut self, keycode: KeyCode, modifiers: KeyMods) {
        let context = get_context();
        context.keys_down.remove(&keycode);

        context
            .input_events
            .push(MiniquadInputEvent::KeyUp { keycode, modifiers });
    }

    fn update(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // TODO: consider making it a part of miniquad?
            std::thread::yield_now();
        }
    }

    fn draw(&mut self) {
        {
            let _z = telemetry::ZoneGuard::new("Event::draw");

            if let Some(future) = unsafe { MAIN_FUTURE.as_mut() } {
                let _z = telemetry::ZoneGuard::new("Main loop");

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

            #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
            {
                let _z = telemetry::ZoneGuard::new("glFinish/glFLush");

                unsafe {
                    miniquad::gl::glFlush();
                    miniquad::gl::glFinish();
                }
            }
        }

        telemetry::reset();
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
                unsafe {
                    CONTEXT = Some(Context::new(ctx))
                };
                UserData::free(Stage {})
            },
        );
    }
}
