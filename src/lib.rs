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

mod cubemap;
mod error;
pub mod shadowmap;

pub use error::Error;

pub mod scene;
pub mod sprite_layer;

pub(crate) mod image;

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

use crate::{
    color::{colors::*, Color},
    quad_gl::QuadGl,
    texture::TextureHandle,
};

use glam::{vec2, Mat4, Vec2};
use std::sync::{Arc, Mutex, Weak};

struct ContextOld {
    screen_width: f32,
    screen_height: f32,

    prevent_quit_event: bool,
    quit_requested: bool,

    cursor_grabbed: bool,

    input_events: Vec<Vec<MiniquadInputEvent>>,

    camera_matrix: Option<Mat4>,

    // ui_context: UiContext,
    coroutines_context: experimental::coroutines::CoroutinesContext,

    start_time: f64,
    last_frame_time: f64,
    frame_time: f64,

    #[cfg(one_screenshot)]
    counter: usize,

    //texture_batcher: texture::Batcher,
    unwind: bool,
    recovery_future: Option<Pin<Box<dyn Future<Output = ()>>>>,

    textures: crate::texture::TexturesContext,
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

impl ContextOld {
    //const DEFAULT_BG_COLOR: Color = BLACK;

    fn new() -> ContextOld {
        let (screen_width, screen_height) = miniquad::window::screen_size();

        unsafe {
            miniquad::gl::glEnable(miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS);
        }
        ContextOld {
            screen_width,
            screen_height,

            prevent_quit_event: false,
            quit_requested: false,

            cursor_grabbed: false,

            input_events: Vec::new(),

            camera_matrix: None,

            // ui_context: UiContext::new(&mut *ctx, screen_width, screen_height),
            // fonts_storage: text::FontsStorage::new(&mut *ctx),
            // texture_batcher: texture::Batcher::new(&mut *ctx),
            coroutines_context: experimental::coroutines::CoroutinesContext::new(),

            start_time: miniquad::date::now(),
            last_frame_time: miniquad::date::now(),
            frame_time: 1.0 / 60.0,

            #[cfg(one_screenshot)]
            counter: 0,
            unwind: false,
            recovery_future: None,

            textures: crate::texture::TexturesContext::new(),
        }
    }

    /// Returns the handle for this texture.
    pub fn raw_miniquad_id(&self, handle: &TextureHandle) -> miniquad::TextureId {
        // match handle {
        //     TextureHandle::Unmanaged(texture) => *texture,
        //     TextureHandle::Managed(texture) => self
        //         .textures
        //         .texture(texture.0)
        //         .unwrap_or(self.white_texture),
        //     TextureHandle::ManagedWeak(texture) => self
        //         .textures
        //         .texture(*texture)
        //         .unwrap_or(self.white_texture),
        // }
        unimplemented!()
    }

    fn end_frame(&mut self) {
        //crate::experimental::scene::update();

        //self.perform_render_passes();

        // self.ui_context.draw(get_quad_ctx(), &mut self.gl);
        // let screen_mat = self.pixel_perfect_projection_matrix();
        // self.gl.draw(get_quad_ctx(), screen_mat);

        //for canvas in self.scene_graph.canvases {}

        #[cfg(one_screenshot)]
        {
            get_context().counter += 1;
            if get_context().counter == 3 {
                crate::prelude::get_screen_data().export_png("screenshot.png");
                panic!("screenshot successfully saved to `screenshot.png`");
            }
        }

        self.quit_requested = false;

        //telemetry::end_gpu_query();
    }

    pub(crate) fn pixel_perfect_projection_matrix(&self) -> glam::Mat4 {
        let (width, height) = miniquad::window::screen_size();

        let dpi = miniquad::window::dpi_scale();

        glam::Mat4::orthographic_rh_gl(0., width / dpi, height / dpi, 0., -1., 1.)
    }

    pub(crate) fn projection_matrix(&self) -> glam::Mat4 {
        if let Some(matrix) = self.camera_matrix {
            matrix
        } else {
            self.pixel_perfect_projection_matrix()
        }
    }
}

#[no_mangle]
static mut CONTEXT: Option<ContextOld> = None;

// This is required for #[macroquad::test]
//
// unfortunately #[cfg(test)] do not work with integration tests
// so this module should be publicly available
#[doc(hidden)]
pub mod test {
    pub static mut MUTEX: Option<std::sync::Mutex<()>> = None;
    pub static ONCE: std::sync::Once = std::sync::Once::new();
}

fn get_context() -> &'static mut ContextOld {
    unsafe { CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

struct Stage {
    main_future: Option<Pin<Box<dyn Future<Output = ()>>>>,
    ctx: Arc<Context3>,
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
        println!("down");
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
        println!("up");

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
	        {
            let _z = telemetry::ZoneGuard::new("Event::draw");

            use std::panic;

            // let scene = scene::Scene {
            //     data: &self.ctx.scene,
            //     ctx: self.ctx.clone(),
            // };
            // {
            //     let _z = telemetry::ZoneGuard::new("Event::draw begin_frame");

            //     scene.clear(Color::new(0.2, 0.2, 0.5, 1.));
            // }

            {
                let _z = telemetry::ZoneGuard::new("clear");

                let mut ctx = self.ctx.quad_ctx.lock().unwrap();
                let clear = PassAction::clear_color(0.2, 0.2, 0.2, 1.);

                ctx.begin_default_pass(clear);
                ctx.end_render_pass();
            }

            //let result = maybe_unwind(get_context().unwind, || {
            if let Some(future) = self.main_future.as_mut() {
                let _z = telemetry::ZoneGuard::new("user code");

                if exec::resume(future).is_some() {
                    self.main_future = None;
                    miniquad::window::quit();
                    return;
                }
                //get_context().coroutines_context.update();
            }
            //});

            {
                let _z = telemetry::ZoneGuard::new("Event::draw end_frame");
                get_context().end_frame();
                self.ctx.input.lock().unwrap().end_frame();
                let mut ctx = self.ctx.quad_ctx.lock().unwrap();
                ctx.commit_frame()
            }
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

    fn window_restored_event(&mut self) {
        #[cfg(target_os = "android")]
        get_context().audio_context.resume();
    }

    fn window_minimized_event(&mut self) {
        #[cfg(target_os = "android")]
        get_context().audio_context.pause();
    }

    fn quit_requested_event(&mut self) {
        let context = get_context();
        if context.prevent_quit_event {
            miniquad::window::cancel_quit();
            context.quit_requested = true;
        }
    }
}

#[derive(Clone)]
pub struct Context3 {
    pub quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    pub(crate) input: Arc<Mutex<input::InputContext>>,
    textures: Arc<Mutex<crate::texture::TexturesContext>>,
    fonts_storage: Arc<Mutex<text::FontsStorage>>,
}

impl Context3 {
    pub(crate) fn new() -> Context3 {
        let mut ctx = miniquad::window::new_rendering_backend();

        let fonts_storage = text::FontsStorage::new(&mut *ctx);
        let textures = crate::texture::TexturesContext::new();
        Context3 {
            quad_ctx: Arc::new(Mutex::new(ctx)),
            fonts_storage: Arc::new(Mutex::new(fonts_storage)),
            textures: Arc::new(Mutex::new(textures)),
            input: Arc::new(Mutex::new(input::InputContext::new())),
        }
    }

    pub fn new_scene(&self) -> scene::Scene {
        scene::Scene::new(self.quad_ctx.clone(), self.fonts_storage.clone())
    }

    pub fn new_canvas(&self) -> sprite_layer::SpriteLayer {
        sprite_layer::SpriteLayer::new(self.quad_ctx.clone(), self.fonts_storage.clone())
    }
}

pub fn start<F: Fn(Context3) -> Fut + 'static, Fut: Future<Output = ()> + 'static>(
    mut config: conf::Conf,
    future: F,
) {
    miniquad::start(conf::Conf { ..config }, move || {
        unsafe { CONTEXT = Some(ContextOld::new()) };
        let ctx = Context3::new();
        Box::new(Stage {
            main_future: Some(Box::pin(future(ctx.clone()))),
            ctx: Arc::new(ctx),
        })
    });
}
