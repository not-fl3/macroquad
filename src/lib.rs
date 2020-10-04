use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;
pub use megaui::hash;

pub use glam::{self, vec2, vec3, Vec2, Vec3};

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

mod camera;
mod drawing;
mod exec;
mod models;
mod shapes;
mod texture;
mod material;
mod time;
mod types;
mod ui;

pub mod collections;
pub mod coroutines;

pub use camera::{Camera, Camera2D, Camera3D, Projection};

pub use macroquad_macro::main;
pub use models::*;
pub use shapes::*;
pub use texture::*;
pub use material::*;
pub use time::*;
pub use types::*;
pub use ui::*;

pub use collections::*;
pub use drawing::FilterMode;
pub use miniquad::{conf::Conf, Comparison, PipelineParams, UniformType};
pub use quad_gl::{colors::*, GlPipeline, QuadGl, Vertex};
pub use quad_rand as rand;

#[cfg(feature = "log-impl")]
pub use miniquad::{debug, error, info, warn};

use drawing::DrawContext;

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
            mouse_position: Vec2::new(0., 0.),
            mouse_wheel: Vec2::new(0., 0.),

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

pub fn next_frame() -> exec::FrameFuture {
    exec::FrameFuture
}

pub use miniquad::{KeyCode, MouseButton};

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

pub fn is_mouse_button_down(btn: MouseButton) -> bool {
    let context = get_context();

    context.mouse_pressed.contains(&btn)
}

pub fn mouse_over_ui() -> bool {
    let context = get_context();

    context.draw_context.ui.is_mouse_over(megaui::Vector2::new(
        context.mouse_position.x(),
        context.mouse_position.y(),
    ))
}

pub fn clear_background(color: Color) {
    let context = get_context();

    // all drawcalls are batched
    // and batching is not clear-friendly
    // so as a workaround we do immediate render pass with clear color
    let clear = PassAction::clear_color(
        color.0[0] as f32 / 255.,
        color.0[1] as f32 / 255.,
        color.0[2] as f32 / 255.,
        color.0[3] as f32 / 255.,
    );
    if let Some(current_pass) = context.draw_context.current_pass {
        context.quad_context.begin_pass(current_pass, clear);
    } else {
        context.quad_context.begin_default_pass(clear);
    }
    context.quad_context.end_render_pass();

    context.draw_context.gl.clear_draw_calls();
}

/// Set active 2D or 3D camera
pub fn set_camera<T: Camera>(camera: T) {
    let context = get_context();

    // flush previous camera draw calls
    context
        .draw_context
        .perform_render_passes(&mut context.quad_context);

    context.draw_context.current_pass = camera.render_pass();
    context.draw_context.gl.render_pass(camera.render_pass());
    context.draw_context.gl.depth_test(camera.depth_enabled());
    context.draw_context.camera_matrix = Some(camera.matrix());
    context
        .draw_context
        .update_projection_matrix(&mut context.quad_context);
}

/// Reset default 2D camera mode
pub fn set_default_camera() {
    let context = get_context();

    // flush previous camera draw calls
    context
        .draw_context
        .perform_render_passes(&mut context.quad_context);

    context.draw_context.current_pass = None;
    context.draw_context.gl.render_pass(None);
    context.draw_context.gl.depth_test(false);
    context.draw_context.camera_matrix = None;
    context
        .draw_context
        .update_projection_matrix(&mut context.quad_context);
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height
}

pub unsafe fn get_internal_gl<'a>() -> &'a mut quad_gl::QuadGl {
    let context = &mut get_context().draw_context;

    &mut context.gl
}

pub fn load_file(path: &str) -> exec::FileLoadingFuture {
    use std::cell::RefCell;
    use std::rc::Rc;

    let contents = Rc::new(RefCell::new(None));
    let path = path.to_owned();

    {
        let contents = contents.clone();
        let err_path = path.clone();

        miniquad::fs::load_file(&path, move |bytes| {
            *contents.borrow_mut() =
                Some(bytes.map_err(|kind| exec::FileError::new(kind, &err_path)));
        });
    }

    exec::FileLoadingFuture { contents }
}
