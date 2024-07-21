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

const FRAGMENT_WITH_ARRAY: &str = r#"#version 100
varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 test_color[10];

void main() {
    gl_FragColor = test_color[5] * texture2D(Texture, uv);
}"#;

#[macroquad::main("Shaders")]
async fn main() {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };

    let mat = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("test_color", UniformType::Float4)],
            pipeline_params,
            ..Default::default()
        },
    )
    .unwrap();

    let mat_with_array = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT_WITH_ARRAY,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::array(
                UniformDesc::new("test_color", UniformType::Float4),
                10,
            )],
            pipeline_params,
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

        gl_use_material(&mat_with_array);
        let mut colors: [Vec4; 10] = [vec4(0.0, 1.0, 0.0, 0.0); 10];
        colors[5] = vec4(0.0, 1.0, 1.0, 1.0);
        mat_with_array.set_uniform_array("test_color", &colors[..]);
        draw_rectangle(50.0, 160.0, 100., 100., WHITE);

        gl_use_default_material();

        draw_rectangle(380.0, 50.0, 100., 100., YELLOW);

        next_frame().await
    }
}
