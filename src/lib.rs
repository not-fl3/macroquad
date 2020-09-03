use miniquad::Context as QuadContext;
use miniquad::*;

#[cfg(feature="megaui")]
pub use megaui;
#[cfg(feature="megaui")]
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
#[cfg(feature="megaui")]
mod ui;

pub use camera::{Camera, Camera2D, Camera3D, Projection};

pub use macroquad_macro::main;
pub use models::*;
pub use shapes::*;
pub use texture::*;
pub use time::*;
pub use types::*;
#[cfg(feature="megaui")]
pub use ui::*;

pub use drawing::FilterMode;
pub use miniquad::{conf::Conf, Comparison, PipelineParams};
pub use quad_gl::{colors::*, GlPipeline, QuadGl, Vertex};
pub use quad_rand as rand;

#[cfg(feature = "log-impl")]
pub use miniquad::{debug, error, info, log, warn};

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

    start_time: f64,
    last_frame_time: f64,
    frame_time: f64,
}

impl Context {
    const DEFAULT_BG_COLOR: Color = BLACK;

    fn new(mut ctx: QuadContext, ui: drawing::UiDrawContext) -> Context {
        let (screen_width, screen_height) = ctx.screen_size();

        Context {
            screen_width,
            screen_height,

            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_position: Vec2::new(0., 0.),
            mouse_wheel: Vec2::new(0., 0.),

            draw_context: DrawContext::new(&mut ctx, ui),

            quad_context: ctx,

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
        let context = get_context();
        context.screen_width = width;
        context.screen_height = height;

        context.draw_context.ui_ctx.resize_event(width, height);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        get_context().mouse_position = Vec2::new(x, y);
        get_context().draw_context.ui_ctx.mouse_motion_event(x, y);
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        let context = get_context();
        context.mouse_wheel.set_x(x);
        context.mouse_wheel.set_y(y);

        context.draw_context.ui_ctx.mouse_wheel_event(x, y);
    }
    fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        get_context().mouse_pressed.insert(btn);
        get_context().draw_context.ui_ctx.mouse_button_down_event(btn, x, y);
    }
    fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        get_context().mouse_pressed.remove(&btn);
        get_context().draw_context.ui_ctx.mouse_button_up_event(btn, x, y);
    }

    fn char_event(&mut self, character: char, modifiers: KeyMods, repeat: bool) {
        get_context().draw_context.ui_ctx.char_event(character, modifiers, repeat);
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
        let context = get_context();
        context.keys_down.insert(keycode);
        if repeat == false {
            context.keys_pressed.insert(keycode);
        }

        context.draw_context.ui_ctx.key_down_event(keycode, modifiers, repeat);
    }
    fn key_up_event(&mut self, keycode: KeyCode, modifiers: KeyMods) {
        let context = get_context();
        context.keys_down.remove(&keycode);

        context.draw_context.ui_ctx.key_up_event(keycode, modifiers);
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        get_context().draw_context.ui_ctx.touch_event(phase, id, x, y);
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
        }

        get_context().end_frame();

        get_context().frame_time = date::now() - get_context().last_frame_time;
        get_context().last_frame_time = date::now();
    }
}

pub struct MacroConfig<T: drawing::DrawableUi> {
    conf: conf::Conf,
    phantom: std::marker::PhantomData<T>
}

impl<T: drawing::DrawableUi> MacroConfig<T> {
    pub fn new(conf: conf::Conf) -> Self {
        Self {
            conf,
            phantom: std::marker::PhantomData
        }
    }

    fn new_ui(&self, ctx: &mut QuadContext) -> T {
        T::new(ctx)
    }
}

#[cfg(not(feature = "custom-ui"))]
impl From<conf::Conf> for MacroConfig<drawing::UiDrawContext> {
    fn from(conf: conf::Conf) -> Self {
        MacroConfig::new(conf)
    }
}

pub struct Window {}

impl Window {
    #[cfg(feature = "custom-ui")]
    pub fn from_config<T: drawing::DrawableUi>(
        config: impl Into<MacroConfig<T>>,
        future: impl Future<Output = ()> + 'static
    ) {
        let mut config = config.into();

        miniquad::start(
            conf::Conf {
                sample_count: 4,
                ..std::mem::take(&mut config.conf)
            },
            move |mut ctx| {
                unsafe {
                    MAIN_FUTURE = Some(Box::pin(future));
                }
                let ui = Box::new(config.new_ui(&mut ctx));
                unsafe { CONTEXT = Some(Context::new(ctx, ui)) };
                UserData::free(Stage {})
            },
        );
    }

    #[cfg(not(feature = "custom-ui"))]
    pub fn from_config(
        config: impl Into<MacroConfig<drawing::UiDrawContext>>,
        future: impl Future<Output = ()> + 'static
    ) {
        let mut config = config.into();

        miniquad::start(
            conf::Conf {
                sample_count: 4,
                ..std::mem::take(&mut config.conf)
            },
            move |mut ctx| {
                unsafe {
                    MAIN_FUTURE = Some(Box::pin(future));
                }
                let ui = config.new_ui(&mut ctx);
                unsafe { CONTEXT = Some(Context::new(ctx, ui)) };
                UserData::free(Stage {})
            },
        );
    }

    #[cfg(not(feature = "custom-ui"))]
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

#[cfg(feature="megaui")]
pub fn mouse_over_ui() -> bool {
    let context = get_context();

    context.draw_context.ui().is_mouse_over(megaui::Vector2::new(
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
    context.draw_context.gl.depth_test(true);
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

#[cfg(feature="custom-ui")]
pub fn custom_ui<T: drawing::DrawableUi, F: FnOnce(&mut QuadContext, &mut T)>(f: F) {
    let Context { draw_context, quad_context, .. } = get_context();
    let ui_any = draw_context.ui_ctx.any_mut();
    f(quad_context, &mut ui_any.downcast_mut().expect("invalid custom_ui type"))
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
