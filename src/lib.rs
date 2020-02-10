use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;
pub use megaui::hash;

pub use glam::Vec2;

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

pub mod rand;

mod drawing;
pub mod exec;

pub use drawing::*;
pub use macroquad_macro::main;

#[cfg(feature = "log-impl")]
pub use miniquad::{debug, info, log, warn};

struct Context {
    quad_context: QuadContext,

    screen_width: f32,
    screen_height: f32,

    keys_pressed: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_wheel: Vec2,

    draw_context: DrawContext,
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
        }
    }

    fn begin_frame(&mut self) {
        self.draw_context.begin_frame();
    }

    fn end_frame(&mut self) {
        self.draw_context.draw_ui(&mut self.quad_context);

        self.draw_context.end_frame(&mut self.quad_context);

        self.quad_context.commit_frame();

        get_context().mouse_wheel = Vec2::new(0., 0.);
    }

    fn clear(&mut self, color: Color) {
        self.draw_context.clear(color);
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

    fn mouse_motion_event(&mut self, x: f32, y: f32, _dx: f32, _dy: f32) {
        use megaui::InputHandler;

        let context = get_context();

        context.mouse_position = Vec2::new(x, y);
        context.draw_context.ui.mouse_move((x, y));
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        let context = get_context();

        context.mouse_wheel.set_x(x);
        context.mouse_wheel.set_y(y);
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

    fn key_down_event(&mut self, keycode: KeyCode, _: KeyMods, _: bool) {
        let context = get_context();
        context.keys_pressed.insert(keycode);
    }

    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        let context = get_context();
        context.keys_pressed.remove(&keycode);
    }

    fn update(&mut self) {}

    fn draw(&mut self) {
        get_context().begin_frame();

        exec::resume(unsafe { MAIN_FUTURE.as_mut().unwrap() });

        get_context().end_frame();
    }
}

pub struct Window {}

impl Window {
    pub fn new(_label: &str, future: impl Future<Output = ()> + 'static) {
        miniquad::start(conf::Conf::default(), |ctx| {
            unsafe {
                MAIN_FUTURE = Some(Box::pin(future));
            }
            unsafe { CONTEXT = Some(Context::new(ctx)) };
            exec::resume(unsafe { MAIN_FUTURE.as_mut().unwrap() });

            UserData::free(Stage {})
        });
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

pub fn clear_background(color: Color) {
    let context = get_context();

    context.clear(color);
}

pub fn set_screen_coordinates(screen_coordinates: ScreenCoordinates) {
    let mut context = get_context();

    context.draw_context.screen_coordinates = screen_coordinates;
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height
}

pub fn load_texture<'a>(path: &str) -> exec::TextureLoadingFuture {
    use std::cell::RefCell;
    use std::rc::Rc;

    let texture = Rc::new(RefCell::new(None));
    let path = path.to_owned();

    {
        let texture = texture.clone();
        let path0 = path.clone();

        miniquad::fs::load_file(&path, move |bytes| {
            let bytes = bytes.unwrap_or_else(|_| panic!("Not such texture: {}", path0));
            *texture.borrow_mut() = Some(load_texture_file_with_format(&bytes[..], None));
        });
    }

    exec::TextureLoadingFuture { texture }
}

pub fn load_texture_from_image(image: &Image) -> Texture2D {
    load_texture_from_rgba8(image.width, image.height, &image.bytes)
}
