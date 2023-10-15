use miniquad::*;

use glam::{vec3, vec4, Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::{camera::Projection, scene::ShadowSplit};

mod debugquad {
    use miniquad::*;

    #[repr(C)]
    struct Vec2 {
        x: f32,
        y: f32,
    }
    #[repr(C)]
    struct Vertex {
        pos: Vec2,
        uv: Vec2,
    }

    pub struct DebugQuad {
        pipeline: Pipeline,
        bindings: Bindings,
    }

    impl DebugQuad {
        pub fn new(ctx: &mut Context) -> DebugQuad {
            #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );

            let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );

            let pixels: [u8; 4 * 4 * 4] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
                0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ];
            let texture = ctx.new_texture_from_rgba8(4, 4, &pixels);

            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer: index_buffer,
                images: vec![texture],
            };
            let source = match ctx.info().backend {
                Backend::OpenGl => ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                Backend::Metal => ShaderSource::Msl {
                    program: shader::METAL,
                },
            };

            let shader = ctx.new_shader(source, shader::meta()).unwrap();

            let pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("in_pos", VertexFormat::Float2),
                    VertexAttribute::new("in_uv", VertexFormat::Float2),
                ],
                shader,
            );

            DebugQuad { pipeline, bindings }
        }

        pub fn draw(&mut self, ctx: &mut Context, texture: &[TextureId]) {
            for i in 0..4 {
                ctx.begin_default_pass(PassAction::Nothing);
                self.bindings.images[0] = texture[i];
                ctx.apply_pipeline(&self.pipeline);
                ctx.apply_bindings(&self.bindings);
                ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    offset: (-0.8 + i as f32 * 0.4, -0.8),
                }));
                ctx.draw(0, 6, 1);
                ctx.end_render_pass();
            }
        }
    }

    mod shader {
        use miniquad::*;

        pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos * 0.3 + offset, 0, 1);
        texcoord = in_uv;
    }"#;

        pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

        pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float2 offset;
    };

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float2 in_uv    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 uv       [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(
      Vertex v [[stage_in]], 
      constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy * 0.3 + uniforms.offset, 0.0, 1.0);
        out.uv = v.in_uv;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return tex.sample(texSmplr, in.uv);
    }"#;

        pub fn meta() -> ShaderMeta {
            ShaderMeta {
                images: vec!["tex".to_string()],
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
                },
            }
        }

        #[repr(C)]
        pub struct Uniforms {
            pub offset: (f32, f32),
        }
    }
}

pub struct ShadowMap {
    pub shadow_pipeline: Pipeline,
    pub shadow_pass: Vec<RenderPass>,

    pub color_img: Vec<TextureId>,
    pub depth_img: Vec<TextureId>,

    pub dbg: debugquad::DebugQuad,
}

impl ShadowMap {
    pub fn new(ctx: &mut Context) -> ShadowMap {
        let mut color_img = vec![];
        let mut depth_img = vec![];
        let mut shadow_pass = vec![];
        for i in 0..4 {
            let c = ctx.new_render_texture(TextureParams {
                width: 4096,
                height: 4096,
                format: TextureFormat::RGBA8,
                ..Default::default()
            });
            let d = ctx.new_render_texture(TextureParams {
                width: 4096,
                height: 4096,
                format: TextureFormat::Depth32,
                ..Default::default()
            });

            color_img.push(c);
            depth_img.push(d);
            shadow_pass.push(ctx.new_render_pass(c, Some(d)));
        }
        let source = match ctx.info().backend {
            Backend::OpenGl => ShaderSource::Glsl {
                vertex: offscreen_shader::VERTEX,
                fragment: offscreen_shader::FRAGMENT,
            },
            Backend::Metal => ShaderSource::Msl {
                program: offscreen_shader::METAL,
            },
        };
        let offscreen_shader = ctx.new_shader(source, offscreen_shader::meta()).unwrap();

        let shadow_pipeline = ctx.new_pipeline_with_params(
            &[
                BufferLayout::default(),
                BufferLayout::default(),
                BufferLayout::default(),
            ],
            &[
                VertexAttribute::with_buffer("in_pos", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("in_uv", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("in_normal", VertexFormat::Float3, 2),
            ],
            offscreen_shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        ShadowMap {
            dbg: debugquad::DebugQuad::new(ctx),
            shadow_pass,
            shadow_pipeline,
            color_img,
            depth_img,
        }
    }

    pub fn draw_shadow_pass(
        &mut self,
        ctx: &mut Context,
        models: &[crate::scene::Model2],
        camera: &crate::camera::Camera,
        shadow_caster: &crate::scene::ShadowCaster,
        clipping_planes: [crate::scene::frustum::Plane; 6],
    ) -> ([Mat4; 4], [f32; 4]) {
        let (width, height) = window::screen_size();

        let split_count = match shadow_caster.split {
            ShadowSplit::Orthogonal => 1,
            ShadowSplit::PSSM2 => 2,
            ShadowSplit::PSSM4 => 4,
        };
        let matrices =
            light_view_porijection_matrices(split_count, &camera, shadow_caster.direction);

        for i in 0..split_count {
            ctx.begin_pass(
                Some(self.shadow_pass[i]),
                PassAction::clear_color(1.0, 1.0, 1.0, 1.0),
            );

            let depth_view_proj = matrices[i];
            for crate::scene::Model2 {
                model,
                transform,
                world_aabb,
            } in models
            {
                if clipping_planes.iter().any(|p| !p.clip(*world_aabb)) {
                    continue;
                }

                for node in &model.nodes {
                    for bindings in &node.data {
                        let model = transform.matrix() * node.transform.matrix();

                        ctx.apply_pipeline(&self.shadow_pipeline);
                        ctx.apply_bindings_from_slice(
                            &bindings.vertex_buffers,
                            bindings.index_buffer,
                            &[],
                        );
                        ctx.apply_uniforms(UniformsSource::table(&offscreen_shader::Uniforms {
                            mvp: depth_view_proj.0 * model,
                        }));
                        let len = ctx.buffer_size(bindings.index_buffer) / 2;
                        ctx.draw(0, len as _, 1);
                    }
                }
            }

            ctx.end_render_pass();
        }

        (
            [matrices[0].0, matrices[1].0, matrices[2].0, matrices[3].0],
            [matrices[0].2, matrices[1].2, matrices[2].2, matrices[3].2],
        )
    }
}

mod offscreen_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_pos;
    attribute vec2 in_uv;
    attribute vec3 in_normal;

    uniform mat4 mvp;

    void main() {
        gl_Position = mvp * vec4(in_pos, 1.0);
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100

    void main() {
        gl_FragColor = vec4(0.0);
    }
    "#;

    pub const METAL: &str = "";

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
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

pub fn light_view_porijection_matrices(
    num_cascades: usize,
    camera: &crate::camera::Camera,
    light_dir: Vec3,
) -> Vec<(Mat4, [Vec4; 8], f32)> {
    let (screen_width, screen_height) = miniquad::window::screen_size();
    let aspect = screen_width / screen_height;
    let mut res = vec![Default::default(); 4];

    let light_view = Mat4::look_at_rh(
        light_dir, // + middle,
        vec3(0.0, 0.0, 0.0), // + middle,
        vec3(0.0, 1.0, 0.0),
    );

    let (proj, view) = camera.proj_view();
    let inv = (proj * view).inverse();
    let mut ndc = [
        vec4(-1.0, -1.0, -1.0, 1.0),
        vec4(1.0, -1.0, -1.0, 1.0),
        vec4(-1.0, 1.0, -1.0, 1.0),
        vec4(1.0, 1.0, -1.0, 1.0),
        vec4(-1.0, -1.0, 1.0, 1.0),
        vec4(1.0, -1.0, 1.0, 1.0),
        vec4(-1.0, 1.0, 1.0, 1.0),
        vec4(1.0, 1.0, 1.0, 1.0),
    ];
    let mut ndc_debug = ndc.clone();
    for ndc in &mut ndc_debug {
        *ndc = inv * *ndc;
        *ndc /= ndc.w;
    }
    for ndc in &mut ndc {
        *ndc = light_view * inv * *ndc;
        *ndc /= ndc.w;
    }

    let cascades_split2 = [0.0, 0.3, 1.0];
    let cascades_split4 = [0.0, 0.1, 0.2, 0.5, 1.0];
    for cascade in 0..num_cascades {
        let near;
        let far;
        if num_cascades == 1 {
            near = 0.0;
            far = 1.0;
        } else if num_cascades == 2 {
            near = cascades_split2[cascade];
            far = cascades_split2[cascade + 1];
        } else if num_cascades == 4 {
            near = cascades_split4[cascade];
            far = cascades_split4[cascade + 1];
        } else {
            panic!()
        };

        let ndc_debug = [
            ndc_debug[0].lerp(ndc_debug[4], near),
            ndc_debug[1].lerp(ndc_debug[5], near),
            ndc_debug[2].lerp(ndc_debug[6], near),
            ndc_debug[3].lerp(ndc_debug[7], near),
            ndc_debug[0].lerp(ndc_debug[4], far),
            ndc_debug[1].lerp(ndc_debug[5], far),
            ndc_debug[2].lerp(ndc_debug[6], far),
            ndc_debug[3].lerp(ndc_debug[7], far),
        ];
        let ndc1 = [
            ndc[0].lerp(ndc[4], near),
            ndc[1].lerp(ndc[5], near),
            ndc[2].lerp(ndc[6], near),
            ndc[3].lerp(ndc[7], near),
            ndc[0].lerp(ndc[4], far),
            ndc[1].lerp(ndc[5], far),
            ndc[2].lerp(ndc[6], far),
            ndc[3].lerp(ndc[7], far),
        ];

        let mut min = vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        let mut max = vec3(-std::f32::MAX, -std::f32::MAX, -std::f32::MAX);

        for ndc in &ndc1 {
            min = min.min(ndc.xyz());
            max = max.max(ndc.xyz());
        }

        let light_proj = Mat4::orthographic_rh_gl(min.x, max.x, min.y, max.y, -max.z, -min.z);
        res[cascade] = (
            light_proj * light_view,
            ndc_debug,
            camera.z_near + far * (camera.z_far - camera.z_near),
        );
    }
    res
}
