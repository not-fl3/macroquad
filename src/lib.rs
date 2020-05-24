use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;
pub use megaui::hash;

pub use glam::{vec2, Vec2};

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

pub mod drawing;
pub mod exec;

mod camera;
mod models;
mod shapes;
mod texture;
mod time;
mod types;
mod ui;

pub use camera::{Camera, Camera2D, Camera3D, Projection};

pub use macroquad_macro::main;
pub use models::*;
pub use shapes::*;
pub use texture::*;
pub use time::*;
pub use types::*;
pub use ui::*;

pub use miniquad::{PipelineParams, Comparison};
pub use quad_gl::{colors::*, QuadGl, Vertex, GlPipeline};
pub use quad_rand as rand;

#[cfg(feature = "log-impl")]
pub use miniquad::{debug, info, log, warn};

use drawing::DrawContext;

struct Context {
    quad_context: QuadContext,

    screen_width: f32,
    screen_height: f32,

    keys_pressed: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_wheel: Vec2,

    draw_context: DrawContext,

    start_time: f64,
    last_frame_time: f64,
    frame_time: f64,
}

impl Context {
    fn new(mut ctx: QuadContext) -> Context {
        let (screen_width, screen_height) = ctx.screen_size();

        Context {
            screen_width,
            screen_height,

            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_position: Vec2::new(0., 0.),
            mouse_wheel: Vec2::new(0., 0.),

            draw_context: DrawContext::new(&mut ctx),

            quad_context: ctx,

            start_time: miniquad::date::now(),
            last_frame_time: miniquad::date::now(),
            frame_time: 1. / 60.,
        }
    }

    fn end_frame(&mut self) {
        self.draw_context
            .perform_render_passes(&mut self.quad_context);

        self.quad_context.commit_frame();

        get_context().mouse_wheel = Vec2::new(0., 0.);
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

    fn char_event(&mut self, character: char, _keymods: KeyMods, _repeat: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context.draw_context.ui.char_event(character);
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, _: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context.keys_pressed.insert(keycode);

        match keycode {
            KeyCode::Up => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Up, modifiers.shift, modifiers.ctrl),
            KeyCode::Down => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Down, modifiers.shift, modifiers.ctrl),
            KeyCode::Right => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Right, modifiers.shift, modifiers.ctrl),
            KeyCode::Left => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Left, modifiers.shift, modifiers.ctrl),
            KeyCode::Home => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Home, modifiers.shift, modifiers.ctrl),
            KeyCode::End => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::End, modifiers.shift, modifiers.ctrl),
            KeyCode::Delete => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Delete, modifiers.shift, modifiers.ctrl),
            KeyCode::Backspace => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Backspace, modifiers.shift, modifiers.ctrl),
            KeyCode::Enter => context
                .draw_context
                .ui
                .key_down(megaui::KeyCode::Enter, modifiers.shift, modifiers.ctrl),
            _ => {}
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        let context = get_context();
        context.keys_pressed.remove(&keycode);
    }

    fn update(&mut self) {}

    fn draw(&mut self) {
        if let Some(future) = unsafe { MAIN_FUTURE.as_mut() } {
            if exec::resume(future) {
                unsafe {
                    MAIN_FUTURE = None;
                }
                info!("macroquad is done");
                get_context().quad_context.quit();
                return;
            }
        }

        get_context().end_frame();

        get_context().frame_time = date::now() - get_context().last_frame_time;
        get_context().last_frame_time = date::now();
    }
}

pub struct Window {}

impl Window {
    pub fn new(label: &str, future: impl Future<Output = ()> + 'static) {
        miniquad::start(
            conf::Conf {
                sample_count: 4,
                window_title: label.to_string(),
                ..Default::default()
            },
            |ctx| {
                unsafe {
                    MAIN_FUTURE = Some(Box::pin(future));
                }
                unsafe { CONTEXT = Some(Context::new(ctx)) };
                if exec::resume(unsafe { MAIN_FUTURE.as_mut().unwrap() }) {
                    unsafe {
                        MAIN_FUTURE = None;
                    }
                }

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

pub fn is_key_down(key_code: KeyCode) -> bool {
    let context = get_context();

    context.keys_pressed.contains(&key_code)
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

    context
        .quad_context
        .begin_default_pass(PassAction::clear_color(
            color.0[0] as f32 / 255.,
            color.0[1] as f32 / 255.,
            color.0[2] as f32 / 255.,
            color.0[3] as f32 / 255.,
        ));
    context.quad_context.end_render_pass();

    context.clear(color);
}

/// Set active 2D or 3D camera
pub fn set_camera<T: Camera>(camera: T) {
    let context = get_context();

    context.draw_context.gl.depth_test(true);
    context.draw_context.camera_matrix = Some(camera.matrix());
    context
        .draw_context
        .update_projection_matrix(&mut context.quad_context);
}

/// Reset default 2D camera mode
pub fn set_default_camera() {
    let context = get_context();

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

pub fn gl_make_pipeline(
    fragment_shader: &str,
    vertex_shader: &str,
    params: PipelineParams,
) -> Result<GlPipeline, ShaderError> {
    let context = &mut get_context();

    context.draw_context.gl.make_pipeline(
        &mut context.quad_context,
        fragment_shader,
        vertex_shader,
        params,
    )
}

pub fn gl_use_pipeline(pipeline: GlPipeline) {
    let context = &mut get_context().draw_context;

    context.gl.pipeline(Some(pipeline));
}

pub fn gl_use_default_pipeline() {
    let context = &mut get_context().draw_context;

    context.gl.pipeline(None);
}
