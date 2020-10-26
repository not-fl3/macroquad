use miniquad::*;

pub use colors::*;

pub use miniquad::{FilterMode, ShaderError};

const UNIFORMS_ARRAY_SIZE: usize = 512;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [
            self.0[0] as f32 / 255.,
            self.0[1] as f32 / 255.,
            self.0[2] as f32 / 255.,
            self.0[3] as f32 / 255.,
        ]
    }
}

impl Color {
    #[rustfmt::skip]
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Color {
        let r;
        let g;
        let b;

        if s == 0.0 {  r = l; g = l; b = l; }
        else {
            fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
                if t < 0.0 { t += 1.0 }
                if t > 1.0 { t -= 1.0 }
                if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
                if t < 1.0 / 2.0 { return q; }
                if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
                return p;
            }

            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;
            r = hue_to_rgb(p, q, h + 1.0 / 3.0);
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - 1.0 / 3.0);
        }

        Color::new(r, g, b, 1.0)
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color([
            (r.min(1.).max(0.) * 255.) as u8,
            (g.min(1.).max(0.) * 255.) as u8,
            (b.min(1.).max(0.) * 255.) as u8,
            (a.min(1.).max(0.) * 255.) as u8,
        ])
    }

    pub fn r(&self) -> f32 {
        self.0[0] as f32 / 255.0
    }

    pub fn g(&self) -> f32 {
        self.0[1] as f32 / 255.0
    }

    pub fn b(&self) -> f32 {
        self.0[2] as f32 / 255.0
    }

    pub fn a(&self) -> f32 {
        self.0[3] as f32 / 255.0
    }
}

pub mod colors {
    use super::Color;

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
}

const MAX_VERTICES: usize = 10000;
const MAX_INDICES: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
    Triangles,
    Lines,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlPipeline(usize);

struct DrawCall {
    vertices: [Vertex; MAX_VERTICES],
    indices: [u16; MAX_INDICES],

    vertices_count: usize,
    indices_count: usize,

    clip: Option<(i32, i32, i32, i32)>,
    texture: Texture,

    model: glam::Mat4,
    projection: glam::Mat4,

    draw_mode: DrawMode,
    pipeline: GlPipeline,
    render_pass: Option<RenderPass>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
    color: Color,
}

pub type VertexInterop = ([f32; 3], [f32; 2], [f32; 4]);

impl Into<VertexInterop> for Vertex {
    fn into(self) -> VertexInterop {
        (
            self.pos,
            self.uv,
            [
                self.color.0[0] as f32 / 255.,
                self.color.0[1] as f32 / 255.,
                self.color.0[2] as f32 / 255.,
                self.color.0[3] as f32 / 255.,
            ],
        )
    }
}
impl Into<Vertex> for VertexInterop {
    fn into(self) -> Vertex {
        Vertex {
            pos: self.0,
            uv: self.1,
            color: Color([
                ((self.2)[0] * 255.) as u8,
                ((self.2)[1] as f32 * 255.) as u8,
                ((self.2)[2] as f32 * 255.) as u8,
                ((self.2)[3] as f32 * 255.) as u8,
            ]),
        }
    }
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: Color) -> Vertex {
        Vertex {
            pos: [x, y, z],
            uv: [u, v],
            color,
        }
    }
}

impl DrawCall {
    fn new(
        texture: Texture,
        projection: glam::Mat4,
        model: glam::Mat4,
        draw_mode: DrawMode,
        pipeline: GlPipeline,
        render_pass: Option<RenderPass>,
    ) -> DrawCall {
        DrawCall {
            vertices: [Vertex::new(0., 0., 0., 0., 0., Color([0, 0, 0, 0])); MAX_VERTICES],
            indices: [0; MAX_INDICES],
            vertices_count: 0,
            indices_count: 0,
            clip: None,
            texture,
            projection,
            model,
            draw_mode,
            pipeline,
            render_pass,
        }
    }

    fn vertices(&self) -> &[Vertex] {
        &self.vertices[0..self.vertices_count]
    }

    fn indices(&self) -> &[u16] {
        &self.indices[0..self.indices_count]
    }
}

struct MagicSnapshoter {
    pipeline: Pipeline,
    bindings: Bindings,
    pass: Option<RenderPass>,

    screen_texture: Option<Texture2D>,
}

mod snapshoter_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;

    varying lowp vec2 uv;

    void main() {
        gl_Position = vec4(position, 0, 1);
        uv = texcoord;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 uv;
    
    uniform sampler2D Texture;

    void main() {
        gl_FragColor = texture2D(Texture, uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout { uniforms: vec![] },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {}
}

impl MagicSnapshoter {
    fn new(ctx: &mut Context) -> MagicSnapshoter {
        let shader = Shader::new(
            ctx,
            snapshoter_shader::VERTEX,
            snapshoter_shader::FRAGMENT,
            snapshoter_shader::meta(),
        )
        .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        #[rustfmt::skip]
        let vertices: [f32; 16] = [
             -1.0, -1.0, 0., 0.,
             1.0, -1.0, 1., 0. ,
             1.0,  1.0, 1., 1. ,
            -1.0,  1.0, 0., 1. ,
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![Texture::empty()],
        };

        MagicSnapshoter {
            pipeline,
            bindings,
            pass: None,
            screen_texture: None,
        }
    }

    fn snapshot(&mut self, ctx: &mut Context, camera_render_pass: Option<RenderPass>) {
        if let Some(camera_render_pass) = camera_render_pass {
            let texture = camera_render_pass.texture(ctx);
            if self.pass.is_none() {
                let color_img = Texture::new_render_texture(
                    ctx,
                    TextureParams {
                        width: texture.width,
                        height: texture.height,
                        format: texture.format,
                        ..Default::default()
                    },
                );

                self.pass = Some(RenderPass::new(ctx, color_img, None));
                self.screen_texture = Some(Texture2D::from_miniquad_texture(color_img));
            }

            if self.bindings.images.len() == 0 {
                self.bindings.images.push(texture);
            } else {
                self.bindings.images[0] = texture;
            }
            ctx.begin_pass(
                self.pass.unwrap(),
                PassAction::clear_color(1.0, 0.0, 1.0, 1.),
            );
            ctx.apply_pipeline(&self.pipeline);
            ctx.apply_bindings(&self.bindings);
            ctx.draw(0, 6, 1);
            ctx.end_render_pass();
        } else {
            let (screen_width, screen_height) = ctx.screen_size();
            if self.screen_texture.is_none()
                || self
                    .screen_texture
                    .map(|t| {
                        t.texture.width != screen_width as _
                            || t.texture.height != screen_height as _
                    })
                    .unwrap_or(false)
            {
                self.screen_texture = Some(Texture2D::from_miniquad_texture(
                    Texture::new_render_texture(
                        ctx,
                        TextureParams {
                            width: screen_width as _,
                            height: screen_height as _,
                            ..Default::default()
                        },
                    ),
                ))
            }

            let texture = self.screen_texture.unwrap();
            texture.grab_screen();
        }
    }
}

struct GlState {
    texture: Texture,
    draw_mode: DrawMode,
    clip: Option<(i32, i32, i32, i32)>,
    projection: glam::Mat4,
    model_stack: Vec<glam::Mat4>,
    pipeline: Option<GlPipeline>,
    depth_test_enable: bool,

    snapshoter: MagicSnapshoter,

    render_pass: Option<RenderPass>,
}

impl GlState {
    fn model(&self) -> glam::Mat4 {
        *self.model_stack.last().unwrap()
    }
}

#[derive(Clone)]
struct Uniform {
    name: String,
    uniform_type: UniformType,
    byte_offset: usize,
}

#[derive(Clone)]
struct PipelineExt {
    pipeline: miniquad::Pipeline,
    wants_screen_texture: bool,
    uniforms: Vec<Uniform>,
    uniforms_data: [u8; UNIFORMS_ARRAY_SIZE],
}

struct PipelinesStorage {
    pipelines: [Option<PipelineExt>; Self::MAX_PIPELINES],
    pipelines_amount: usize,
}

impl PipelinesStorage {
    const MAX_PIPELINES: usize = 32;
    const TRIANGLES_PIPELINE: GlPipeline = GlPipeline(0);
    const LINES_PIPELINE: GlPipeline = GlPipeline(1);
    const TRIANGLES_DEPTH_PIPELINE: GlPipeline = GlPipeline(2);
    const LINES_DEPTH_PIPELINE: GlPipeline = GlPipeline(3);

    fn new(ctx: &mut miniquad::Context) -> PipelinesStorage {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta())
            .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let params = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let mut storage = PipelinesStorage {
            pipelines: Default::default(),
            pipelines_amount: 0,
        };

        let triangles_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                ..params
            },
            false,
            vec![],
        );
        assert_eq!(triangles_pipeline, Self::TRIANGLES_PIPELINE);

        let lines_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Lines,
                ..params
            },
            false,
            vec![],
        );
        assert_eq!(lines_pipeline, Self::LINES_PIPELINE);

        let triangles_depth_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                primitive_type: PrimitiveType::Triangles,
                ..params
            },
            false,
            vec![],
        );
        assert_eq!(triangles_depth_pipeline, Self::TRIANGLES_DEPTH_PIPELINE);

        let lines_depth_pipeline = storage.make_pipeline(
            ctx,
            shader,
            PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                primitive_type: PrimitiveType::Lines,
                ..params
            },
            false,
            vec![],
        );
        assert_eq!(lines_depth_pipeline, Self::LINES_DEPTH_PIPELINE);

        storage
    }

    fn make_pipeline(
        &mut self,
        ctx: &mut Context,
        shader: Shader,
        params: PipelineParams,
        wants_screen_texture: bool,
        uniforms: Vec<(String, UniformType)>,
    ) -> GlPipeline {
        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float3),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Byte4),
            ],
            shader,
            params,
        );

        let id = self
            .pipelines
            .iter()
            .position(|p| p.is_none())
            .unwrap_or_else(|| panic!("Pipelines amount exceeded"));

        let uniforms = uniforms
            .iter()
            .scan(0, |offset, uniform| {
                let uniform_byte_size = uniform.1.size();
                if *offset + uniform_byte_size > UNIFORMS_ARRAY_SIZE {
                    println!(
                        "Material exceeds maximum uniforms amount, uniforms after {} skipped",
                        uniform.0
                    );
                    return None;
                }
                let uniform = Uniform {
                    name: uniform.0.clone(),
                    uniform_type: uniform.1,
                    byte_offset: *offset,
                };
                *offset += uniform_byte_size;

                Some(uniform)
            })
            .collect();
        self.pipelines[id] = Some(PipelineExt {
            pipeline,
            wants_screen_texture,
            uniforms,
            uniforms_data: [0; UNIFORMS_ARRAY_SIZE],
        });
        self.pipelines_amount += 1;

        GlPipeline(id)
    }

    fn get(&self, draw_mode: DrawMode, depth_enabled: bool) -> GlPipeline {
        match (draw_mode, depth_enabled) {
            (DrawMode::Triangles, false) => Self::TRIANGLES_PIPELINE,
            (DrawMode::Triangles, true) => Self::TRIANGLES_DEPTH_PIPELINE,
            (DrawMode::Lines, false) => Self::LINES_PIPELINE,
            (DrawMode::Lines, true) => Self::LINES_DEPTH_PIPELINE,
        }
    }

    fn get_quad_pipeline(&self, pip: GlPipeline) -> &PipelineExt {
        &self.pipelines[pip.0].as_ref().unwrap()
    }

    fn get_quad_pipeline_mut(&mut self, pip: GlPipeline) -> &mut PipelineExt {
        self.pipelines[pip.0].as_mut().unwrap()
    }

    fn delete_pipeline(&mut self, pip: GlPipeline) {
        self.pipelines[pip.0] = None;
    }
}

pub struct QuadGl {
    pipelines: PipelinesStorage,

    draw_calls: Vec<DrawCall>,
    draw_calls_bindings: Vec<Bindings>,
    draw_calls_count: usize,
    state: GlState,
    start_time: f64,

    white_texture: Texture,
}

impl QuadGl {
    pub fn new(ctx: &mut miniquad::Context) -> QuadGl {
        let white_texture = Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]);

        QuadGl {
            pipelines: PipelinesStorage::new(ctx),
            state: GlState {
                clip: None,
                texture: white_texture,
                projection: glam::Mat4::identity(),
                model_stack: vec![glam::Mat4::identity()],
                draw_mode: DrawMode::Triangles,
                pipeline: None,
                depth_test_enable: false,
                snapshoter: MagicSnapshoter::new(ctx),
                render_pass: None,
            },
            draw_calls: Vec::with_capacity(200),
            draw_calls_bindings: Vec::with_capacity(200),
            draw_calls_count: 0,
            start_time: miniquad::date::now(),
            white_texture,
        }
    }

    pub fn make_pipeline(
        &mut self,
        ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
        params: PipelineParams,
        uniforms: Vec<(String, UniformType)>,
    ) -> Result<GlPipeline, ShaderError> {
        let mut shader_meta: ShaderMeta = shader::meta();

        for uniform in &uniforms {
            shader_meta
                .uniforms
                .uniforms
                .push(UniformDesc::new(&uniform.0, uniform.1));
        }

        let shader = Shader::new(ctx, vertex_shader, fragment_shader, shader_meta)?;
        let wants_screen_texture = fragment_shader.find("_ScreenTexture").is_some();

        Ok(self
            .pipelines
            .make_pipeline(ctx, shader, params, wants_screen_texture, uniforms))
    }

    /// Reset only draw calls state
    pub fn clear_draw_calls(&mut self) {
        self.draw_calls_count = 0;
    }

    /// Reset internal state to known default
    pub fn reset(&mut self) {
        self.state.clip = None;
        self.state.texture = self.white_texture;
        self.state.projection = glam::Mat4::identity();
        self.state.model_stack = vec![glam::Mat4::identity()];

        self.draw_calls_count = 0;
    }

    pub fn draw(&mut self, ctx: &mut miniquad::Context) {
        for _ in 0..self.draw_calls.len() - self.draw_calls_bindings.len() {
            let vertex_buffer = Buffer::stream(
                ctx,
                BufferType::VertexBuffer,
                MAX_VERTICES * std::mem::size_of::<Vertex>(),
            );
            let index_buffer = Buffer::stream(
                ctx,
                BufferType::IndexBuffer,
                MAX_INDICES * std::mem::size_of::<u16>(),
            );
            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![Texture::empty(), Texture::empty()],
            };

            self.draw_calls_bindings.push(bindings);
        }
        assert_eq!(self.draw_calls_bindings.len(), self.draw_calls.len());

        let (screen_width, screen_height) = ctx.screen_size();
        let time = (miniquad::date::now() - self.start_time) as f32;
        let time = glam::vec4(time, time.sin(), time.cos(), 0.);

        for (dc, bindings) in self.draw_calls[0..self.draw_calls_count]
            .iter_mut()
            .zip(self.draw_calls_bindings.iter_mut())
        {
            let pipeline = self.pipelines.get_quad_pipeline(dc.pipeline);

            let (width, height) = if let Some(render_pass) = dc.render_pass {
                let render_texture = render_pass.texture(ctx);

                (render_texture.width, render_texture.height)
            } else {
                (screen_width as u32, screen_height as u32)
            };

            if pipeline.wants_screen_texture {
                self.state.snapshoter.snapshot(ctx, dc.render_pass);
            }

            if let Some(render_pass) = dc.render_pass {
                ctx.begin_pass(render_pass, PassAction::Nothing);
            } else {
                ctx.begin_default_pass(PassAction::Nothing);
            }

            bindings.vertex_buffers[0].update(ctx, dc.vertices());
            bindings.index_buffer.update(ctx, dc.indices());

            bindings.images[0] = dc.texture;
            bindings.images[1] = self.state.snapshoter.screen_texture.map_or_else(
                || Texture::empty(),
                |texture| texture.raw_miniquad_texture_handle(),
            );

            ctx.apply_pipeline(&pipeline.pipeline);
            if let Some(clip) = dc.clip {
                ctx.apply_scissor_rect(clip.0, height as i32 - (clip.1 + clip.3), clip.2, clip.3);
            } else {
                ctx.apply_scissor_rect(0, 0, width as i32, height as i32);
            }
            ctx.apply_bindings(&bindings);

            ctx.apply_uniforms(&shader::Uniforms {
                projection: dc.projection,
                model: dc.model,
                time,
                data: pipeline.uniforms_data.clone(),
            });
            ctx.draw(0, dc.indices_count as i32, 1);

            dc.vertices_count = 0;
            dc.indices_count = 0;

            ctx.end_render_pass();
        }

        self.draw_calls_count = 0;
    }

    pub fn get_projection_matrix(&self) -> glam::Mat4 {
        self.state.projection
    }

    pub fn get_active_render_pass(&self) -> Option<RenderPass> {
        self.state.render_pass
    }

    pub fn render_pass(&mut self, render_pass: Option<RenderPass>) {
        self.state.render_pass = render_pass;
    }

    pub fn depth_test(&mut self, enable: bool) {
        self.state.depth_test_enable = enable;
    }

    pub fn texture(&mut self, texture: Option<Texture2D>) {
        self.state.texture = texture.map_or(self.white_texture, |t| t.texture);
    }

    pub fn scissor(&mut self, clip: Option<(i32, i32, i32, i32)>) {
        self.state.clip = clip;
    }

    pub fn set_projection_matrix(&mut self, matrix: glam::Mat4) {
        self.state.projection = matrix;
    }

    pub fn push_model_matrix(&mut self, matrix: glam::Mat4) {
        self.state.model_stack.push(self.state.model() * matrix);
    }

    pub fn pop_model_matrix(&mut self) {
        if self.state.model_stack.len() > 1 {
            self.state.model_stack.pop();
        }
    }

    pub fn pipeline(&mut self, pipeline: Option<GlPipeline>) {
        self.state.pipeline = pipeline;
    }

    pub fn draw_mode(&mut self, mode: DrawMode) {
        self.state.draw_mode = mode;
    }

    pub fn geometry(&mut self, vertices: &[impl Into<VertexInterop> + Copy], indices: &[u16]) {
        let pip = self.state.pipeline.unwrap_or(
            self.pipelines
                .get(self.state.draw_mode, self.state.depth_test_enable),
        );

        let previous_dc_ix = if self.draw_calls_count == 0 {
            None
        } else {
            Some(self.draw_calls_count - 1)
        };
        let previous_dc = previous_dc_ix.and_then(|ix| self.draw_calls.get(ix));

        if previous_dc.map_or(true, |draw_call| {
            draw_call.texture != self.state.texture
                || draw_call.clip != self.state.clip
                || draw_call.model != self.state.model()
                || draw_call.pipeline != pip
                || draw_call.render_pass != self.state.render_pass
                || draw_call.draw_mode != self.state.draw_mode
                || draw_call.projection != self.state.projection
                || draw_call.vertices_count >= MAX_VERTICES - vertices.len()
                || draw_call.indices_count >= MAX_INDICES - indices.len()
        }) {
            if self.draw_calls_count >= self.draw_calls.len() {
                self.draw_calls.push(DrawCall::new(
                    self.state.texture,
                    self.state.projection,
                    self.state.model(),
                    self.state.draw_mode,
                    pip,
                    self.state.render_pass,
                ));
            }
            self.draw_calls[self.draw_calls_count].texture = self.state.texture;
            self.draw_calls[self.draw_calls_count].vertices_count = 0;
            self.draw_calls[self.draw_calls_count].indices_count = 0;
            self.draw_calls[self.draw_calls_count].clip = self.state.clip;
            self.draw_calls[self.draw_calls_count].projection = self.state.projection;
            self.draw_calls[self.draw_calls_count].model = self.state.model();
            self.draw_calls[self.draw_calls_count].pipeline = pip;
            self.draw_calls[self.draw_calls_count].render_pass = self.state.render_pass;

            self.draw_calls_count += 1;
        };
        let dc = &mut self.draw_calls[self.draw_calls_count - 1];

        for i in 0..vertices.len() {
            dc.vertices[dc.vertices_count + i] = vertices[i].into().into();
        }

        for i in 0..indices.len() {
            dc.indices[dc.indices_count + i] = indices[i] + dc.vertices_count as u16;
        }
        dc.vertices_count += vertices.len();
        dc.indices_count += indices.len();
        dc.texture = self.state.texture;
    }

    pub fn delete_pipeline(&mut self, pipeline: GlPipeline) {
        self.pipelines.delete_pipeline(pipeline);
    }

    pub fn set_uniform<T>(&mut self, pipeline: GlPipeline, name: &str, uniform: T) {
        let pipeline = self.pipelines.get_quad_pipeline_mut(pipeline);

        let uniform_meta = pipeline.uniforms.iter().find(
            |Uniform {
                 name: uniform_name, ..
             }| uniform_name == name,
        );
        if uniform_meta.is_none() {
            println!("Trying to set non-existing uniform: {}", name);
            return;
        }
        let uniform_meta = uniform_meta.unwrap();
        let uniform_format = uniform_meta.uniform_type;
        let uniform_byte_size = uniform_format.size();
        let uniform_byte_offset = uniform_meta.byte_offset;

        if std::mem::size_of::<T>() != uniform_byte_size {
            println!(
                "Trying to set uniform {} sized {} bytes value of {} bytes",
                name,
                std::mem::size_of::<T>(),
                uniform_byte_size
            );
            return;
        }
        macro_rules! transmute_uniform {
            ($uniform_size:expr, $byte_offset:expr, $n:expr) => {
                if $uniform_size == $n {
                    let data: [u8; $n] = unsafe { std::mem::transmute_copy(&uniform) };

                    for i in 0..$uniform_size {
                        pipeline.uniforms_data[$byte_offset + i] = data[i];
                    }
                }
            };
        }
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 4);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 8);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 12);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 16);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 64);
    }
}

/// Texture, data stored in GPU memory
#[derive(Clone, Copy, Debug)]
pub struct Texture2D {
    texture: miniquad::Texture,
}

impl Texture2D {
    pub fn from_miniquad_texture(texture: miniquad::Texture) -> Texture2D {
        Texture2D { texture }
    }

    pub fn empty() -> Texture2D {
        Texture2D {
            texture: miniquad::Texture::empty(),
        }
    }

    pub fn update(&mut self, ctx: &mut miniquad::Context, image: &Image) {
        assert_eq!(self.texture.width, image.width as u32);
        assert_eq!(self.texture.height, image.height as u32);

        self.texture.update(ctx, &image.bytes);
    }

    pub fn width(&self) -> f32 {
        self.texture.width as f32
    }

    pub fn height(&self) -> f32 {
        self.texture.height as f32
    }

    pub fn from_file_with_format<'a>(
        ctx: &mut miniquad::Context,
        bytes: &[u8],
        format: Option<image::ImageFormat>,
    ) -> Texture2D {
        let img = if let Some(fmt) = format {
            image::load_from_memory_with_format(&bytes, fmt)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba()
        } else {
            image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba()
        };
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Self::from_rgba8(ctx, width, height, &bytes)
    }

    pub fn from_rgba8(
        ctx: &mut miniquad::Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> Texture2D {
        let texture = miniquad::Texture::from_rgba8(ctx, width, height, &bytes);

        Texture2D { texture }
    }

    pub fn set_filter(&self, ctx: &mut miniquad::Context, filter_mode: FilterMode) {
        self.texture.set_filter(ctx, filter_mode);
    }

    pub fn raw_miniquad_texture_handle(&self) -> Texture {
        self.texture
    }

    pub fn grab_screen(&self) {
        let (internal_format, _, _) = self.texture.format.into();
        unsafe {
            gl::glBindTexture(gl::GL_TEXTURE_2D, self.texture.gl_internal_id());
            gl::glCopyTexImage2D(
                gl::GL_TEXTURE_2D,
                0,
                internal_format,
                0,
                0,
                self.texture.width as _,
                self.texture.height as _,
                0,
            );
        }
    }

    pub fn get_texture_data(&self) -> Image {
        let mut image = Image {
            width: self.texture.width as _,
            height: self.texture.height as _,
            bytes: vec![0; self.texture.width as usize * self.texture.height as usize * 4],
        };

        self.texture.read_pixels(&mut image.bytes);

        image
    }
}

/// Image, data stored in CPU memory
pub struct Image {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl Image {
    pub fn from_file_with_format(bytes: &[u8], format: Option<image::ImageFormat>) -> Image {
        let img = if let Some(fmt) = format {
            image::load_from_memory_with_format(&bytes, fmt)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba()
        } else {
            image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba()
        };
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Image {
            width,
            height,
            bytes,
        }
    }

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

    pub fn get_image_data(&self) -> &[Color] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                self.bytes.as_ptr() as *const Color,
                self.width as usize * self.height as usize,
            )
        }
    }

    pub fn get_image_data_mut(&mut self) -> &mut [Color] {
        use std::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.bytes.as_mut_ptr() as *mut Color,
                self.width as usize * self.height as usize,
            )
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let width = self.width;

        self.get_image_data_mut()[(y * width as u32 + x) as usize] = color;
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        self.get_image_data()[(y * self.width as u32 + x) as usize]
    }

    pub fn export_png(&self, path: &str) {
        let mut bytes = vec![0; self.width as usize * self.height as usize * 4];

        // flip the image before saving
        for y in 0..self.height as usize {
            for x in 0..self.width as usize * 4 {
                bytes[y * self.width as usize * 4 + x] =
                    self.bytes[(self.height as usize - y - 1) * self.width as usize * 4 + x];
            }
        }

        image::save_buffer(
            path,
            &bytes[..],
            self.width as _,
            self.height as _,
            image::ColorType::RGBA(8),
        )
        .unwrap();
    }
}

mod shader {
    use super::UNIFORMS_ARRAY_SIZE;
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
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

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string(), "_ScreenTexture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("_Time", UniformType::Float4),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: glam::Mat4,
        pub model: glam::Mat4,
        pub time: glam::Vec4,

        pub data: [u8; UNIFORMS_ARRAY_SIZE],
    }
}
