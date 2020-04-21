use miniquad::Context as QuadContext;
use miniquad::*;

pub use megaui;
pub use megaui::hash;

pub use glam::{vec2, Vec2};

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

pub mod rand;

pub mod drawing;
pub mod exec;

mod camera;
mod time;

pub use camera::Camera2D;

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

        get_context().frame_time = date::now() - get_context().last_frame_time;
        get_context().last_frame_time = date::now();
    }
}

pub struct Window {}

impl Window {
    pub fn new(_label: &str, future: impl Future<Output = ()> + 'static) {
        miniquad::start(
            conf::Conf {
                sample_count: 4,
                ..Default::default()
            },
            |ctx| {
                unsafe {
                    MAIN_FUTURE = Some(Box::pin(future));
                }
                unsafe { CONTEXT = Some(Context::new(ctx)) };
                exec::resume(unsafe { MAIN_FUTURE.as_mut().unwrap() });

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

pub fn begin_mode_2d(camera: Camera2D) {
    let context = get_context();

    assert!(
        context.draw_context.camera_2d.is_none(),
        "2d drawing mode already in progress"
    );

    context.draw_context.camera_2d = Some(camera);
    context
        .draw_context
        .update_projection_matrix(&mut context.quad_context);
}

pub fn end_mode_2d() {
    let context = get_context();

    assert!(
        context.draw_context.camera_2d.is_some(),
        "Not in 2D rendering mode"
    );

    context.draw_context.camera_2d = None;
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

pub fn set_texture_filter(texture: Texture2D, filter_mode: FilterMode) {
    let context = &mut get_context().quad_context;

    texture.set_filter(context, filter_mode);
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

    let atlas = context.ui.font_atlas.clone();

    let mut total_width = 0.;
    for character in text.chars() {
        if let Some(font_data) = atlas.character_infos.get(&character) {
            let font_data = font_data.scale(font_size);

            total_width += font_data.left_padding;

            let left_coord = total_width;
            let top_coord = atlas.font_size as f32 - font_data.height_over_line;

            total_width += font_data.size.0 + font_data.right_padding;

            let dest = Rect::new(
                left_coord + x,
                top_coord + y - 5.,
                font_data.size.0,
                font_data.size.1,
            );

            let source = Rect::new(
                font_data.tex_coords.0 * context.font_texture.width(),
                font_data.tex_coords.1 * context.font_texture.height(),
                font_data.tex_size.0 * context.font_texture.width(),
                font_data.tex_size.1 * context.font_texture.height(),
            );
            draw_texture_ex(
                context.font_texture,
                dest.x,
                dest.y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(dest.w, dest.h)),
                    source: Some(source),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_rectangle(x, y, w, h, color);
}

pub fn draw_texture(texture: Texture2D, x: f32, y: f32, color: Color) {
    draw_texture_ex(texture, x, y, color, Default::default());
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
}

pub struct DrawTextureParams {
    pub dest_size: Option<Vec2>,

    /// Part of texture to draw. If None - draw the whole texture.
    /// Good use example: drawing an image from texture atlas.
    /// Is None by default
    pub source: Option<Rect>,

    /// Rotation in degrees
    pub rotation: f32,
}

impl Default for DrawTextureParams {
    fn default() -> DrawTextureParams {
        DrawTextureParams {
            dest_size: None,
            source: None,
            rotation: 0.,
        }
    }
}

pub fn draw_texture_ex(
    texture: Texture2D,
    x: f32,
    y: f32,
    color: Color,
    params: DrawTextureParams,
) {
    let context = &mut get_context().draw_context;

    let Rect {
        x: sx,
        y: sy,
        w: sw,
        h: sh,
    } = params.source.unwrap_or(Rect {
        x: 0.,
        y: 0.,
        w: texture.width(),
        h: texture.height(),
    });

    let (w, h) = params
        .dest_size
        .map_or((texture.width(), texture.height()), |dst| {
            (dst.x(), dst.y())
        });

    let m = vec2(x + w / 2., y + h / 2.);
    let p = [
        vec2(-w / 2., -h / 2.),
        vec2(w / 2., -h / 2.),
        vec2(w / 2., h / 2.),
        vec2(-w / 2., h / 2.),
    ];
    let r = params.rotation;
    let p = [
        vec2(
            p[0].x() * r.cos() - p[0].y() * r.sin(),
            p[0].x() * r.sin() + p[0].y() * r.cos(),
        ) + m,
        vec2(
            p[1].x() * r.cos() - p[1].y() * r.sin(),
            p[1].x() * r.sin() + p[1].y() * r.cos(),
        ) + m,
        vec2(
            p[2].x() * r.cos() - p[2].y() * r.sin(),
            p[2].x() * r.sin() + p[2].y() * r.cos(),
        ) + m,
        vec2(
            p[3].x() * r.cos() - p[3].y() * r.sin(),
            p[3].x() * r.sin() + p[3].y() * r.cos(),
        ) + m,
    ];
    #[rustfmt::skip]
    let vertices = [
        Vertex::new(p[0].x(), p[0].y(), 0.,  sx      /texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[1].x(), p[1].y(), 0., (sx + sw)/texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[2].x(), p[2].y(), 0., (sx + sw)/texture.width(), (sy + sh)/texture.height(), color),
        Vertex::new(p[3].x(), p[3].y(), 0.,  sx      /texture.width(), (sy + sh)/texture.height(), color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.gl.texture(Some(texture));
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let t = thickness / 2.;

    draw_rectangle(x, y, w, t, color);
    draw_rectangle(x + w - t, y + t, t, h - t, color);
    draw_rectangle(x, y + h - t, w, t, color);
    draw_rectangle(x, y + t, t, h - t, color);
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

#[deprecated(since = "0.3.0", note = "Use draw_texture_ex instead")]
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
    draw_texture_ex(
        texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            source: Some(Rect {
                x: sx,
                y: sy,
                w: sw,
                h: sh,
            }),
            ..Default::default()
        },
    );
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    let context = &mut get_context().draw_context;

    const NUM_DIVISIONS: u32 = 200;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertices.push(Vertex::new(x, y, 0., 0., 0., color));
    for i in 0..NUM_DIVISIONS + 1 {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, color);

        vertices.push(vertex);

        if i != NUM_DIVISIONS {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.gl.texture(None);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    const NUM_DIVISIONS: u32 = 200;

    for i in 0..NUM_DIVISIONS {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let p0 = vec2(x + r * rx, y + r * ry);

        let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let p1 = vec2(x + r * rx, y + r * ry);

        draw_line(p0.x(), p0.y(), p1.x(), p1.y(), thickness, color);
    }
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = &mut get_context().draw_context;
    context.draw_line(x1, y1, x2, y2, thickness, color);
}

pub struct WindowParams {
    pub label: String,
    pub movable: bool,
    pub close_button: bool,
}
impl Default for WindowParams {
    fn default() -> WindowParams {
        WindowParams {
            label: "".to_string(),
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
    .label(params.as_ref().map_or("", |params| &params.label))
    .movable(params.as_ref().map_or(true, |params| params.movable))
    .close_button(params.as_ref().map_or(false, |params| params.close_button))
    .ui(&mut context.ui, f)
}
