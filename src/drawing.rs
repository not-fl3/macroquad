use miniquad::*;

use crate::get_context;

const FONT_TEXTURE_BYTES: &'static [u8] = include_bytes!("font.png");
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

pub const LIGHTGRAY: Color = Color([200, 200, 200, 255]);
pub const GRAY: Color = Color([130, 130, 130, 255]);
pub const DARKGRAY: Color = Color([80, 80, 80, 255]);
pub const YELLOW: Color = Color([253, 249, 0, 255]);
pub const GOLD: Color = Color([255, 203, 0, 255]);
pub const ORANGE: Color = Color([255, 161, 0, 255]);
pub const PINK: Color = Color([255, 109, 194, 255]);
pub const RED: Color = Color([230, 41, 55, 255]);
pub const MAROON: Color = Color([190, 33, 55, 255]);
pub const GREEN: Color = Color([0, 228, 48, 255]);
pub const LIME: Color = Color([0, 158, 47, 255]);
pub const DARKGREEN: Color = Color([0, 117, 44, 255]);
pub const SKYBLUE: Color = Color([102, 191, 255, 255]);
pub const BLUE: Color = Color([0, 121, 241, 255]);
pub const DARKBLUE: Color = Color([0, 82, 172, 255]);
pub const PURPLE: Color = Color([200, 122, 255, 255]);
pub const VIOLET: Color = Color([135, 60, 190, 255]);
pub const DARKPURPLE: Color = Color([112, 31, 126, 255]);
pub const BEIGE: Color = Color([211, 176, 131, 255]);
pub const BROWN: Color = Color([127, 106, 79, 255]);
pub const DARKBROWN: Color = Color([76, 63, 47, 255]);
pub const WHITE: Color = Color([255, 255, 255, 255]);
pub const BLACK: Color = Color([0, 0, 0, 255]);
pub const BLANK: Color = Color([0, 0, 0, 0]);
pub const MAGENTA: Color = Color([255, 0, 255, 255]);

const MAX_VERTICES: usize = 10000;
const MAX_INDICES: usize = 5000;

struct DrawCall {
    vertices: [Vertex; MAX_VERTICES],
    indices: [u16; MAX_INDICES],

    vertices_count: usize,
    indices_count: usize,

    clip: Option<(i32, i32, i32, i32)>,
    texture: Texture,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    u: f32,
    v: f32,
    color: Color,
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, color: Color) -> Vertex {
        Vertex { x, y, u, v, color }
    }
}

impl DrawCall {
    fn new(texture: Texture) -> DrawCall {
        DrawCall {
            vertices: [Vertex::new(0., 0., 0., 0., Color([0, 0, 0, 0])); MAX_VERTICES],
            indices: [0; MAX_INDICES],
            vertices_count: 0,
            indices_count: 0,
            clip: None,
            texture,
        }
    }

    fn vertices(&self) -> &[Vertex] {
        &self.vertices[0..self.vertices_count]
    }

    fn indices(&self) -> &[u16] {
        &self.indices[0..self.indices_count]
    }
}

pub enum ScreenCoordinates {
    Fixed(f32, f32, f32, f32),
    PixelPerfect,
}

pub struct DrawContext {
    pipeline: Pipeline,

    clear_color: Color,
    draw_calls: Vec<DrawCall>,
    draw_calls_bindings: Vec<Bindings>,
    draw_calls_count: usize,
    clip: Option<(i32, i32, i32, i32)>,

    white_texture: Texture,
    font_texture: Texture,
    pub(crate) screen_coordinates: ScreenCoordinates,
    pub(crate) ui: megaui::Ui,
    ui_draw_list: Vec<megaui::DrawCommand>,
}

impl DrawContext {
    pub fn new(ctx: &mut miniquad::Context) -> DrawContext {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::META);

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Byte4),
            ],
            shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let img = image::load_from_memory(FONT_TEXTURE_BYTES)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        let font_texture = Texture::from_rgba8(ctx, width, height, &bytes);
        let white_texture = Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]);

        DrawContext {
            pipeline,
            clear_color: BLACK,
            clip: None,
            screen_coordinates: ScreenCoordinates::PixelPerfect,
            draw_calls: Vec::with_capacity(200),
            draw_calls_bindings: Vec::with_capacity(200),
            draw_calls_count: 0,

            font_texture,
            white_texture,
            ui: megaui::Ui::new(Default::default()),
            ui_draw_list: Vec::with_capacity(10000),
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.draw_calls_count = 0;
    }

    pub fn begin_frame(&mut self) {
        self.draw_calls_count = 0;
    }

    pub fn end_frame(&mut self, ctx: &mut miniquad::Context) {
        for _ in 0..self.draw_calls.len() - self.draw_calls_bindings.len() {
            let vertex_buffer = Buffer::stream(
                ctx,
                BufferType::VertexBuffer,
                MAX_VERTICES * std::mem::size_of::<f32>(),
            );
            let index_buffer = Buffer::stream(
                ctx,
                BufferType::IndexBuffer,
                MAX_INDICES * std::mem::size_of::<u16>(),
            );
            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer: index_buffer,
                images: vec![],
            };

            self.draw_calls_bindings.push(bindings);
        }
        assert_eq!(self.draw_calls_bindings.len(), self.draw_calls.len());

        ctx.begin_default_pass(PassAction::clear_color(
            self.clear_color.0[0] as f32 / 255.0,
            self.clear_color.0[1] as f32 / 255.0,
            self.clear_color.0[2] as f32 / 255.0,
            self.clear_color.0[3] as f32 / 255.0,
        ));

        let (width, height) = ctx.screen_size();

        let projection = match self.screen_coordinates {
            ScreenCoordinates::PixelPerfect => {
                glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.)
            }
            ScreenCoordinates::Fixed(left, right, bottom, top) => {
                glam::Mat4::orthographic_rh_gl(left, right, bottom, top, -1., 1.)
            }
        };

        for (dc, bindings) in self.draw_calls[0..self.draw_calls_count]
            .iter_mut()
            .zip(self.draw_calls_bindings.iter_mut())
        {
            bindings.vertex_buffers[0].update(ctx, dc.vertices());
            bindings.index_buffer.update(ctx, dc.indices());
            bindings.images = vec![dc.texture];

            ctx.apply_pipeline(&self.pipeline);
            if let Some(clip) = dc.clip {
                ctx.apply_scissor_rect(clip.0, height as i32 - (clip.1 + clip.3), clip.2, clip.3);
            } else {
                ctx.apply_scissor_rect(0, 0, width as i32, height as i32);
            }
            ctx.apply_bindings(&bindings);
            ctx.apply_uniforms(&shader::Uniforms { projection });
            ctx.draw(0, dc.indices_count as i32, 1);

            dc.vertices_count = 0;
            dc.indices_count = 0;
        }

        ctx.end_render_pass();
    }

    fn draw_call(&mut self, vertices: &[Vertex], indices: &[u16], texture: Texture) {
        let previous_dc_ix = if self.draw_calls_count == 0 {
            None
        } else {
            Some(self.draw_calls_count - 1)
        };
        let previous_dc = previous_dc_ix.and_then(|ix| self.draw_calls.get(ix));

        if previous_dc.map_or(true, |draw_call| {
            draw_call.texture != texture
                || draw_call.clip != self.clip
                || draw_call.vertices_count >= MAX_VERTICES - vertices.len()
                || draw_call.indices_count >= MAX_INDICES - indices.len()
        }) {
            if self.draw_calls_count >= self.draw_calls.len() {
                self.draw_calls.push(DrawCall::new(texture));
            }
            self.draw_calls[self.draw_calls_count].texture = texture;
            self.draw_calls[self.draw_calls_count].vertices_count = 0;
            self.draw_calls[self.draw_calls_count].indices_count = 0;
            self.draw_calls[self.draw_calls_count].clip = self.clip;

            self.draw_calls_count += 1;
        };
        let dc = &mut self.draw_calls[self.draw_calls_count - 1];

        for i in 0..vertices.len() {
            dc.vertices[dc.vertices_count + i] = vertices[i];
        }

        for i in 0..indices.len() {
            dc.indices[dc.indices_count + i] = indices[i] + dc.vertices_count as u16;
        }
        dc.vertices_count += vertices.len();
        dc.indices_count += indices.len();
        dc.texture = texture;
    }

    pub fn draw_ui(&mut self, _: &mut miniquad::Context) {
        self.ui_draw_list.clear();

        self.ui.render(&mut self.ui_draw_list);
        self.ui.new_frame();

        for draw_command in &self.ui_draw_list {
            use megaui::DrawCommand::*;

            match draw_command {
                Clip {
                    rect: Some(megaui::Rect { x, y, w, h }),
                } => self.clip = Some((*x as i32, *y as i32, *w as i32, *h as i32)),
                Clip { rect: None } => self.clip = None,
                DrawLabel {
                    params,
                    position,
                    label,
                } => {
                    let color = params.color;

                    draw_text(
                        label,
                        position.x,
                        position.y,
                        10.,
                        Color([
                            (color.r * 255.) as u8,
                            (color.g * 255.) as u8,
                            (color.b * 255.) as u8,
                            (color.a * 255.) as u8,
                        ]),
                    );
                }
                DrawRect { rect, stroke, fill } => {
                    if let Some(fill) = fill {
                        draw_rectangle(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            Color([
                                (fill.r * 255.) as u8,
                                (fill.g * 255.) as u8,
                                (fill.b * 255.) as u8,
                                (fill.a * 255.) as u8,
                            ]),
                        );
                    }
                    if let Some(stroke) = stroke {
                        draw_rectangle_lines(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            Color([
                                (stroke.r * 255.) as u8,
                                (stroke.g * 255.) as u8,
                                (stroke.b * 255.) as u8,
                                (stroke.a * 255.) as u8,
                            ]),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// Texture, data stored in GPU memory
#[derive(Clone, Copy, Debug)]
pub struct Texture2D {
    texture: miniquad::Texture,
}

impl Texture2D {
    pub fn empty() -> Texture2D {
        Texture2D {
            texture: miniquad::Texture::empty(),
        }
    }

    pub fn update(&mut self, image: &Image) {
        assert_eq!(self.texture.width, image.width as u32);
        assert_eq!(self.texture.height, image.height as u32);

        let context = &mut get_context();
        let quad_ctx = context.quad_context.as_mut().unwrap_or_else(|| panic!());
        self.texture.update(quad_ctx, &image.bytes);
    }

    pub fn width(&self) -> f32 {
        self.texture.width as f32
    }

    pub fn height(&self) -> f32 {
        self.texture.height as f32
    }
}

pub(crate) fn load_texture_file_with_format<'a>(
    bytes: &[u8],
    format: Option<image::ImageFormat>,
) -> Texture2D {
    let img = if let Some(fmt) = format {
        image::load_from_memory_with_format(&bytes, fmt)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba()
    } else {
        image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba()
    };
    let width = img.width() as u16;
    let height = img.height() as u16;
    let bytes = img.into_raw();

    load_texture_from_rgba8(width, height, &bytes)
}

pub(crate) fn load_texture_from_rgba8(width: u16, height: u16, bytes: &[u8]) -> Texture2D {
    let context = get_context();

    let texture = miniquad::Texture::from_rgba8(
        context.quad_context.as_mut().unwrap_or_else(|| panic!()),
        width,
        height,
        &bytes,
    );

    Texture2D { texture }
}

/// Image, data stored in CPU memory
pub struct Image {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl Image {
    pub fn empty() -> Image {
        Image {
            width: 0,
            height: 0,
            bytes: vec![],
        }
    }

    pub fn gen_image_color(width: u16, height: u16, color: Color) -> Image {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        for i in 0..width as usize * height as usize {
            for c in 0..4 {
                bytes[i * 4 + c] = color.0[c];
            }
        }
        Image {
            width,
            height,
            bytes,
        }
    }

    pub fn update(&mut self, bytes: &[Color]) {
        assert!(self.width as usize * self.height as usize == bytes.len());

        for i in 0..bytes.len() {
            self.bytes[i * 4] = bytes[i].0[0];
            self.bytes[i * 4 + 1] = bytes[i].0[1];
            self.bytes[i * 4 + 2] = bytes[i].0[2];
            self.bytes[i * 4 + 3] = bytes[i].0[3];
        }
    }
    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn get_image_data(&mut self) -> &mut [Color] {
        use std::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.bytes.as_mut_ptr() as *mut Color,
                self.width as usize * self.height as usize,
            )
        }
    }
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let context = &mut get_context().draw_context;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();
    for (n, ch) in text.chars().enumerate() {
        let ix = ch as u32;

        let sx = ((ix % 16) as f32) / 16.0;
        let sy = ((ix / 16) as f32) / 16.0;
        let sw = 1.0 / 16.0;
        let sh = 1.0 / 16.0;

        #[rustfmt::skip]
        let letter = [
            Vertex::new(x + 0.0 + n as f32 * font_size, y, sx, sy, color),
            Vertex::new(x + font_size + n as f32 * font_size, y, sx + sw, sy, color),
            Vertex::new(x + font_size + n as f32 * font_size, y + font_size, sx + sw, sy + sh, color),
            Vertex::new(x + 0.0 + n as f32 * font_size, y + font_size, sx, sy + sh, color),
        ];
        vertices.extend(letter.iter());
        let n = n as u16;
        indices.extend(
            [
                n * 4 + 0,
                n * 4 + 1,
                n * 4 + 2,
                n * 4 + 0,
                n * 4 + 2,
                n * 4 + 3,
            ]
            .iter()
            .map(|x| *x),
        );
    }

    context.draw_call(&vertices, &indices, context.font_texture);
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , 0.0, 1.0, color),
        Vertex::new(x + w, y    , 1.0, 0.0, color),
        Vertex::new(x + w, y + h, 1.0, 1.0, color),
        Vertex::new(x    , y + h, 0.0, 0.0, color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.draw_call(&vertices, &indices, context.white_texture);
}

pub fn draw_texture(texture: Texture2D, x: f32, y: f32, color: Color) {
    let context = &mut get_context().draw_context;

    let w = texture.texture.width as f32;
    let h = texture.texture.height as f32;

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , 0.0, 0.0, color),
        Vertex::new(x + w, y    , 1.0, 0.0, color),
        Vertex::new(x + w, y + h, 1.0, 1.0, color),
        Vertex::new(x    , y + h, 0.0, 1.0, color),
        ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.draw_call(&vertices, &indices, texture.texture);
}

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x, y, w, 1., color);
    draw_rectangle(x + w - 1., y + 1., 1., h - 2., color);
    draw_rectangle(x, y + h - 1., w, 1., color);
    draw_rectangle(x, y + 1., 1., h - 2., color);
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

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , sx     , sy     , color),
        Vertex::new(x + w, y    , sx + sw, sy     , color),
        Vertex::new(x + w, y + h, sx + sw, sy + sh, color),
        Vertex::new(x    , y + h, sx     , sy + sh, color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.draw_call(&vertices, &indices, texture.texture);
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    const NUM_DIVISIONS: u32 = 20;

    let context = &mut get_context().draw_context;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertices.push(Vertex::new(x, y, 0., 0.0, color));
    for i in 0..NUM_DIVISIONS + 1 {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let vertex = Vertex::new(x + r * rx, y + r * ry, rx, ry, color);

        vertices.push(vertex);

        if i != NUM_DIVISIONS {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.draw_call(&vertices, &indices, context.white_texture);
}

pub fn draw_window<F: FnOnce(&mut megaui::Ui)>(
    id: megaui::Id,
    position: glam::Vec2,
    size: glam::Vec2,
    f: F,
) {
    let context = &mut get_context().draw_context;

    megaui::widgets::Window::new(
        id,
        megaui::Vector2::new(position.x(), position.y()),
        megaui::Vector2::new(size.x(), size.y()),
    )
    .ui(&mut context.ui, f);
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;
    
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * vec4(position, 0, 1);
        gl_Position.z = 0.;
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;
    
    uniform sampler2D Texture;

    void main() {
        gl_FragColor = color * texture2D(Texture, uv) ;
    }"#;

    pub const META: ShaderMeta = ShaderMeta {
        images: &["Texture"],
        uniforms: UniformBlockLayout {
            uniforms: &[("Projection", UniformType::Mat4)],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: glam::Mat4,
    }
}
