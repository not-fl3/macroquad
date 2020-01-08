use miniquad::Context as QuadContext;
use miniquad::*;

const MAX_VERTICES: usize = 10000;
const MAX_INDICES: usize = 5000;

const FONT_TEXTURE_BYTES: &'static [u8] = include_bytes!("font.png");

struct DrawCall {
    vertices: [f32; MAX_VERTICES],
    indices: [u16; MAX_INDICES],

    vertices_count: usize,
    indices_count: usize,

    texture: Texture,
}

impl DrawCall {
    fn new(texture: Texture) -> DrawCall {
        DrawCall {
            vertices: [0.0; MAX_VERTICES],
            indices: [0; MAX_INDICES],
            vertices_count: 0,
            indices_count: 0,
            texture,
        }
    }

    fn vertices(&self) -> &[f32] {
        &self.vertices[0..self.vertices_count]
    }

    fn indices(&self) -> &[u16] {
        &self.indices[0..self.indices_count]
    }
}

struct Context {
    clear_color: Color,

    screen_width: f32,
    screen_height: f32,

    pipeline: Pipeline,

    white_texture: Texture,
    font_texture: Texture,

    draw_calls: Vec<DrawCall>,
    draw_calls_bindings: Vec<Bindings>,
    draw_calls_count: usize,
}

impl Context {
    fn new(ctx: &mut QuadContext) -> Context {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::META);

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Float4),
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

        let (screen_width, screen_height) = ctx.screen_size();

        let img = image::load_from_memory(FONT_TEXTURE_BYTES)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        let font_texture = Texture::from_rgba8(width, height, &bytes);

        let white_texture = Texture::from_rgba8(1, 1, &[255, 255, 255, 255]);

        Context {
            clear_color: Color(0., 0., 0., 1.),
            pipeline,

            screen_width,
            screen_height,

            font_texture,
            white_texture,

            draw_calls: Vec::with_capacity(200),
            draw_calls_bindings: Vec::with_capacity(200),
            draw_calls_count: 0,
        }
    }

    fn begin_frame(&mut self, _ctx: &mut QuadContext) {
        self.draw_calls_count = 0;
    }

    fn end_frame(&mut self, ctx: &mut QuadContext) {
        for _ in 0..self.draw_calls.len() - self.draw_calls_bindings.len() {
            let vertex_buffer = Buffer::stream(ctx, BufferType::VertexBuffer, MAX_VERTICES);
            let index_buffer = Buffer::stream(ctx, BufferType::IndexBuffer, MAX_INDICES);
            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer: index_buffer,
                images: vec![],
            };

            self.draw_calls_bindings.push(bindings);
        }
        assert_eq!(self.draw_calls_bindings.len(), self.draw_calls.len());

        ctx.begin_default_pass(PassAction::clear_color(
            self.clear_color.0,
            self.clear_color.1,
            self.clear_color.2,
            self.clear_color.3,
        ));

        let (width, height) = ctx.screen_size();

        let projection = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);

        for (dc, bindings) in self.draw_calls[0..self.draw_calls_count]
            .iter_mut()
            .zip(self.draw_calls_bindings.iter_mut())
        {
            unsafe { bindings.vertex_buffers[0].update(ctx, dc.vertices()) };
            unsafe { bindings.index_buffer.update(ctx, dc.indices()) };
            bindings.images = vec![dc.texture];
            ctx.apply_pipeline(&self.pipeline);
            ctx.apply_bindings(&bindings);
            unsafe {
                ctx.apply_uniforms(&shader::Uniforms { projection });
            }
            ctx.draw(0, dc.indices_count as i32, 1);

            dc.vertices_count = 0;
            dc.indices_count = 0;
        }

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.draw_calls_count = 0;
    }

    fn draw_call(&mut self, vertices: &[f32], indices: &[u16], texture: Texture) {
        let previous_dc_ix = if self.draw_calls_count == 0 {
            None
        } else {
            Some(self.draw_calls_count - 1)
        };
        let previous_dc = previous_dc_ix.and_then(|ix| self.draw_calls.get(ix));

        if previous_dc.map_or(true, |draw_call| draw_call.texture != texture) {
            if self.draw_calls_count >= self.draw_calls.len() {
                self.draw_calls.push(DrawCall::new(texture));
            }
            self.draw_calls[self.draw_calls_count].texture = texture;
            self.draw_calls[self.draw_calls_count].vertices_count = 0;
            self.draw_calls[self.draw_calls_count].indices_count = 0;

            self.draw_calls_count += 1;
        };
        let dc = &mut self.draw_calls[self.draw_calls_count - 1];

        dc.texture = texture;
        for i in 0..vertices.len() {
            dc.vertices[dc.vertices_count + i] = vertices[i];
        }

        for i in 0..indices.len() {
            dc.indices[dc.indices_count + i] = indices[i] + dc.vertices_count as u16 / (2 + 2 + 4);
        }
        dc.vertices_count += vertices.len();
        dc.indices_count += indices.len();
        dc.texture = texture;
    }
}

static mut CONTEXT: Option<Context> = None;

fn get_context() -> &'static mut Context {
    unsafe { CONTEXT.as_mut().unwrap() }
}

struct Stage {
    f: Box<dyn Fn() -> ()>,
}

impl EventHandler for Stage {
    fn resize_event(&mut self, _ctx: &mut QuadContext, width: f32, height: f32) {
        let context = get_context();

        context.screen_width = width;
        context.screen_height = height;
    }

    fn update(&mut self, _ctx: &mut QuadContext) {}

    fn draw(&mut self, ctx: &mut QuadContext) {
        get_context().begin_frame(ctx);

        (self.f)();

        get_context().end_frame(ctx);
    }
}

pub struct Window {}

impl Window {
    pub fn init(_label: &str) -> Window {
        Window {}
    }

    pub fn main_loop(self, f: impl Fn() -> () + 'static) {
        miniquad::start(conf::Conf::default(), move |ctx| {
            unsafe { CONTEXT = Some(Context::new(ctx)) };

            Box::new(Stage { f: Box::new(f) })
        });
    }
}

#[repr(C)]
pub struct Color(f32, f32, f32, f32);

pub const RED: Color = Color(1.0, 0.0, 0.0, 1.0);
pub const DARKGRAY: Color = Color(0.8, 0.8, 0.8, 1.0);
pub const GREEN: Color = Color(0.0, 1.0, 0.0, 1.0);
pub const YELLOW: Color = Color(1.0, 1.0, 0.0, 1.0);

pub fn clear_background(color: Color) {
    let context = get_context();

    context.clear(color);
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let context = get_context();

    let mut vertices = Vec::<f32>::new();
    let mut indices = Vec::<u16>::new();
    for (n, ch) in text.chars().enumerate() {
        let ix = ch as u32;

        let sx = ((ix % 16) as f32) / 16.0;
        let sy = ((ix / 16) as f32) / 16.0;
        let sw = 1.0 / 16.0;
        let sh = 1.0 / 16.0;

        #[rustfmt::skip]
        let letter = [
            x + 0.0 + n as f32 * font_size, y, sx, sy, color.0, color.1, color.2, color.3,
            x + font_size + n as f32 * font_size, y, sx + sw, sy, color.0, color.1, color.2, color.3,
            x + font_size + n as f32 * font_size, y + font_size, sx + sw, sy + sh, color.0, color.1, color.2, color.3,
            x + 0.0 + n as f32 * font_size, y + font_size, sx, sy + sh, color.0, color.1, color.2, color.3,
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
    let context = get_context();

    #[rustfmt::skip]
    let vertices = [
        x    , y    , 0.0, 1.0, color.0, color.1, color.2, color.3,
        x + w, y    , 1.0, 0.0, color.0, color.1, color.2, color.3,
        x + w, y + h, 1.0, 1.0, color.0, color.1, color.2, color.3,
        x    , y + h, 0.0, 1.0, color.0, color.1, color.2, color.3,
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.draw_call(&vertices, &indices, context.white_texture);
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    const NUM_DIVISIONS: u32 = 20;

    let context = get_context();

    let mut vertices = Vec::<f32>::new();
    let mut indices = Vec::<u16>::new();

    vertices.extend_from_slice(&[x, y, 0., 0.0, color.0, color.1, color.2, color.3]);
    for i in 0..NUM_DIVISIONS + 1 {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        #[rustfmt::skip]
        let vertex = [x + r * rx, y + r * ry, rx, ry, color.0, color.1, color.2, color.3];

        vertices.extend_from_slice(&vertex);

        if i != NUM_DIVISIONS {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.draw_call(&vertices, &indices, context.white_texture);
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
        color = color0;
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
