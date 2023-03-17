use macroquad::prelude::*;

use macroquad::window::miniquad::*;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 test_color;

void main() {
    gl_FragColor = test_color * texture2D(Texture, uv);
}"#;

#[macroquad::main("Shaders")]
async fn main() {
    let mat = load_material(
        ShaderSource {
            glsl_vertex: Some(VERTEX),
            glsl_fragment: Some(FRAGMENT),
            metal_shader: None,
        },
        MaterialParams {
            uniforms: vec![("test_color".to_string(), UniformType::Float4)],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();

    loop {
        clear_background(GRAY);

        gl_use_material(&mat);

        mat.set_uniform("test_color", vec4(1., 0., 0., 1.));

        draw_rectangle(50.0, 50.0, 100., 100., WHITE);

        mat.set_uniform("test_color", vec4(0., 1., 0., 1.));
        draw_rectangle(160.0, 50.0, 100., 100., WHITE);

        mat.set_uniform("test_color", vec4(0., 0., 1., 1.));
        draw_rectangle(270.0, 50.0, 100., 100., WHITE);

        gl_use_default_material();

        draw_rectangle(380.0, 50.0, 100., 100., YELLOW);

        next_frame().await
    }
}
