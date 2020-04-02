use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;
pub use megaui::hash;

pub use glam::Vec2;

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

pub mod rand;

pub mod drawing;
pub mod exec;

mod time;

pub use time::*;

pub use drawing::*;
pub use macroquad_macro::main;

pub use quad_gl::{QuadGl, Vertex};

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

    start_time: f64,
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

    fn char_event(&mut self, character: char, _keymods: KeyMods, _repeat: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context.draw_context.ui.char_event(character);
    }

    fn key_down_event(&mut self, keycode: KeyCode, _: KeyMods, _: bool) {
        use megaui::InputHandler;

        let context = get_context();
        context.keys_pressed.insert(keycode);

        match keycode {
            KeyCode::Up => context.draw_context.ui.key_down(megaui::KeyCode::Up),
            KeyCode::Down => context.draw_context.ui.key_down(megaui::KeyCode::Down),
            KeyCode::Right => context.draw_context.ui.key_down(megaui::KeyCode::Right),
            KeyCode::Left => context.draw_context.ui.key_down(megaui::KeyCode::Left),
            KeyCode::Home => context.draw_context.ui.key_down(megaui::KeyCode::Home),
            KeyCode::End => context.draw_context.ui.key_down(megaui::KeyCode::End),
            KeyCode::Delete => context.draw_context.ui.key_down(megaui::KeyCode::Delete),
            KeyCode::Backspace => context.draw_context.ui.key_down(megaui::KeyCode::Backspace),
            KeyCode::Enter => context.draw_context.ui.key_down(megaui::KeyCode::Enter),
            _ => {}
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        let context = get_context();
        context.keys_pressed.remove(&keycode);
    }

    fn update(&mut self) {}

    fn draw(&mut self) {
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

pub fn load_texture(path: &str) -> exec::TextureLoadingFuture {
    use std::cell::RefCell;
    use std::rc::Rc;

    let texture = Rc::new(RefCell::new(None));
    let path = path.to_owned();

    {
        let texture = texture.clone();
        let path0 = path.clone();

        miniquad::fs::load_file(&path, move |bytes| {
            let bytes = bytes.unwrap_or_else(|_| panic!("Not such texture: {}", path0));
            let context = &mut get_context().quad_context;

            *texture.borrow_mut() =
                Some(Texture2D::from_file_with_format(context, &bytes[..], None));
        });
    }

    exec::TextureLoadingFuture { texture }
}

/// Upload image data to GPU texture
pub fn update_texture(mut texture: Texture2D, image: &Image) {
    let context = &mut get_context().quad_context;

    texture.update(context, image);
}

pub fn load_texture_from_image(image: &Image) -> Texture2D {
    let context = &mut get_context().quad_context;

    Texture2D::from_rgba8(context, image.width, image.height, &image.bytes)
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_text(text, x, y, font_size, color);
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_rectangle(x, y, w, h, color);
}

pub fn draw_texture(texture: Texture2D, x: f32, y: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_texture(texture, x, y, color);
}

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_rectangle_lines(x, y, w, h, color);
}

pub fn draw_hexagon(
    x: f32,
    y: f32,
    size: f32,
    border: f32,
    border_color: Color,
    fill_color: Color,
) {
    let context = &mut get_context().draw_context;

    const NUM_DIVISIONS: u32 = 6;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertices.push(Vertex::new(x, y, 0., 0., 0., fill_color));
    let mut n = 0;
    for i in 0..NUM_DIVISIONS {
        let d = std::f32::consts::PI / 2.;
        // internal vertices
        {
            let r = size - border;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, fill_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, fill_color);
            vertices.push(vertex);

            indices.extend_from_slice(&[0, n + 1, n + 2]);
        }

        // duplicate internal vertices with border_color
        {
            let r = size - border;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);
        }

        // external border
        {
            let r = size;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);
        }

        indices.extend_from_slice(&[n + 5, n + 3, n + 4]);
        indices.extend_from_slice(&[n + 5, n + 4, n + 6]);

        n += 6;
    }

    context.gl.texture(None);
    context.gl.geometry(&vertices, &indices);
}

pub unsafe fn get_internal_gl<'a>() -> &'a mut quad_gl::QuadGl {
    let context = &mut get_context().draw_context;

    &mut context.gl
}

/// Draw texture to x y w h position on the screen, using sx sy sw sh as a texture coordinates.
/// Good use example: drawing an image from texture atlas.
///
/// TODO: maybe introduce Rect type?
pub fn draw_texture_rec(
    texture: Texture2D,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    color: Color,
) {
    let context = &mut get_context().draw_context;
    context.draw_texture_rec(texture, x, y, w, h, sx, sy, sw, sh, color);
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    let context = &mut get_context().draw_context;
    context.draw_circle(x, y, r, color);
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = &mut get_context().draw_context;
    context.draw_line(x1, y1, x2, y2, thickness, color);
}

pub struct WindowParams {
    pub movable: bool,
    pub close_button: bool,
}
impl Default for WindowParams {
    fn default() -> WindowParams {
        WindowParams {
            movable: true,
            close_button: false,
        }
    }
}

pub fn set_ui_style(style: megaui::Style) {
    get_context().draw_context.ui.set_style(style);
}

pub fn draw_window<F: FnOnce(&mut megaui::Ui)>(
    id: megaui::Id,
    position: glam::Vec2,
    size: glam::Vec2,
    params: impl Into<Option<WindowParams>>,
    f: F,
) -> bool {
    let context = &mut get_context().draw_context;
    let params = params.into();

    megaui::widgets::Window::new(
        id,
        megaui::Vector2::new(position.x(), position.y()),
        megaui::Vector2::new(size.x(), size.y()),
    )
    .movable(params.as_ref().map_or(true, |params| params.movable))
    .close_button(params.as_ref().map_or(false, |params| params.movable))
    .ui(&mut context.ui, f)
}
