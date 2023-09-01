use miniquad::*;

use glam::{vec3, Mat3, Mat4};
use nanoimage::png;

#[derive(Debug)]
pub struct Cubemap {
    display_pipeline: Pipeline,
    display_bind: Bindings,
    rx: f32,
    ry: f32,
    pub texture: TextureId,
}

impl Cubemap {
    pub fn new(ctx: &mut dyn RenderingBackend, bytes: &[&[u8]]) -> Cubemap {
        let texture0 = png::decode(&bytes[0]).unwrap();
        let texture1 = png::decode(&bytes[1]).unwrap();
        let texture2 = png::decode(&bytes[2]).unwrap();
        let texture3 = png::decode(&bytes[3]).unwrap();
        let texture4 = png::decode(&bytes[4]).unwrap();
        let texture5 = png::decode(&bytes[5]).unwrap();
        let color_img = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Array(&[
                &[&texture0.data],
                &[&texture1.data],
                &[&texture2.data],
                &[&texture3.data],
                &[&texture4.data],
                &[&texture5.data],
            ]),
            TextureParams {
                width: texture0.width as _,
                height: texture0.height as _,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        #[rustfmt::skip]
        let vertices: &[f32] = &[
            /* pos               color                   uvs */
            -1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0, -1.0,    1.0, 0.5, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 0.0,
             1.0, -1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     1.0, 1.0,
            -1.0,  1.0,  1.0,    0.5, 1.0, 0.5, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0,  1.0, -1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 0.0,
            -1.0,  1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     1.0, 1.0,
            -1.0, -1.0,  1.0,    0.5, 0.5, 1.0, 1.0,     0.0, 1.0,

             1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 0.0,
             1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     1.0, 1.0,
             1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,     0.0, 1.0,

            -1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 0.0,
            -1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 0.0,
             1.0, -1.0,  1.0,    0.0, 0.5, 1.0, 1.0,     1.0, 1.0,
             1.0, -1.0, -1.0,    0.0, 0.5, 1.0, 1.0,     0.0, 1.0,

            -1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 0.0,
            -1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 0.0,
             1.0,  1.0,  1.0,    1.0, 0.0, 0.5, 1.0,     1.0, 1.0,
             1.0,  1.0, -1.0,    1.0, 0.0, 0.5, 1.0,     0.0, 1.0
        ];

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        #[rustfmt::skip]
        let indices: &[u16] = &[
            0, 1, 2,  0, 2, 3,
            6, 5, 4,  7, 6, 4,
            8, 9, 10,  8, 10, 11,
            14, 13, 12,  15, 14, 12,
            16, 17, 18,  16, 18, 19,
            22, 21, 20,  23, 22, 20
        ];

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let display_bind = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![color_img],
        };

        let info = ctx.info();
        let default_shader = ctx
            .new_shader(
                match info.backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: display_shader::VERTEX,
                        fragment: display_shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: display_shader::METAL,
                    },
                },
                display_shader::meta(),
            )
            .unwrap();

        let display_pipeline = ctx.new_pipeline_with_params(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            default_shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: false,
                ..Default::default()
            },
        );

        Cubemap {
            display_pipeline,
            display_bind,
            rx: 0.,
            ry: 0.,
            texture: color_img,
        }
    }
}

impl Cubemap {
    pub fn draw(&mut self, ctx: &mut dyn RenderingBackend, proj: &Mat4, view: &Mat4) {
        let (width, height) = window::screen_size();

        let view_proj = *proj * Mat4::from_mat3(Mat3::from_mat4(*view));

        self.rx += 0.01;
        self.ry += 0.03;
        // let model = Mat4::from_rotation_ypr(self.rx, self.ry, 0.);

        let vs_params = display_shader::Uniforms { mvp: view_proj };

        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.display_pipeline);
        ctx.apply_bindings(&self.display_bind);
        ctx.apply_uniforms(UniformsSource::table(&vs_params));
        ctx.draw(0, 36, 1);
        ctx.end_render_pass();
    }
}

mod display_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec4 in_pos;
    attribute vec4 in_color;
    attribute vec2 in_uv;

    varying lowp vec4 color;
    varying lowp vec3 uv;

    uniform mat4 mvp;

    void main() {
        vec4 pos = mvp * in_pos;
        gl_Position = pos.xyww;
        color = in_color;
        uv = in_pos.xyz;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec3 uv;

    uniform samplerCube tex;

    void main() {
        gl_FragColor = textureCube(tex, uv);
    }
    "#;

    pub const METAL: &str = r#"#include <metal_stdlib>
    using namespace metal;

    struct Uniforms
    {
        float4x4 mvp;
    };

    struct Vertex
    {
        float3 in_pos      [[attribute(0)]];
        float4 in_color    [[attribute(1)]];
        float2 in_uv       [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.mvp * float4(v.in_pos, 1.0);
        out.color = v.in_color;
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
