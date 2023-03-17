use macroquad::prelude::*;

#[macroquad::main("Raw miniquad")]
async fn main() {
    let stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        raw_miniquad::Stage::new(ctx)
    };

    loop {
        clear_background(LIGHTGRAY);

        // Render some primitives in camera space

        set_camera(&Camera2D {
            zoom: vec2(1., screen_width() / screen_height()),
            ..Default::default()
        });
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

        {
            let mut gl = unsafe { get_internal_gl() };

            // Ensure that macroquad's shapes are not going to be lost
            gl.flush();

            let t = get_time();

            gl.quad_context.apply_pipeline(&stage.pipeline);

            gl.quad_context
                .begin_default_pass(miniquad::PassAction::Nothing);
            gl.quad_context.apply_bindings(&stage.bindings);

            for i in 0..10 {
                let t = t + i as f64 * 0.3;

                gl.quad_context
                    .apply_uniforms(miniquad::UniformsSource::table(
                        &raw_miniquad::shader::Uniforms {
                            offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
                        },
                    ));
                gl.quad_context.draw(0, 6, 1);
            }
            gl.quad_context.end_render_pass();
        }

        // Back to screen space, render some text

        set_default_camera();
        draw_text("HELLO", 30.0, 200.0, 30.0, BLACK);

        next_frame().await
    }
}

mod raw_miniquad {
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

    pub struct Stage {
        pub pipeline: Pipeline,
        pub bindings: Bindings,
    }

    impl Stage {
        pub fn new(ctx: &mut dyn RenderingBackend) -> Stage {
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
                BufferSource::slice(&indices[..]),
            );

            let pixels: [u8; 4 * 4 * 4] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
                0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ];
            let texture = ctx.new_texture_from_rgba8(4, 4, &pixels);

            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer,
                images: vec![texture],
            };

            let shader = ctx
                .new_shader(
                    miniquad::ShaderSource {
                        glsl_vertex: Some(shader::VERTEX),
                        glsl_fragment: Some(shader::FRAGMENT),
                        metal_shader: None,
                    },
                    shader::meta(),
                )
                .unwrap();

            let pipeline = ctx.new_pipeline(
                &[BufferLayout::default()],
                &[
                    VertexAttribute::new("pos", VertexFormat::Float2),
                    VertexAttribute::new("uv", VertexFormat::Float2),
                ],
                shader,
            );

            Stage { pipeline, bindings }
        }
    }

    pub mod shader {
        use miniquad::*;

        pub const VERTEX: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;

uniform vec2 offset;

varying lowp vec2 texcoord;

void main() {
    gl_Position = vec4(pos + offset, 0, 1);
    texcoord = uv;
}"#;

        pub const FRAGMENT: &str = r#"#version 100
varying lowp vec2 texcoord;

uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, texcoord);
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
