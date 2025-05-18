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

pub const METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct Vertex
{
    float3 position    [[attribute(0)]];
    float2 texcoord    [[attribute(1)]];
    float4 color0      [[attribute(2)]];
};

struct RasterizerData
{
    float4 position [[position]];
    float4 color [[user(locn0)]];
    float2 uv [[user(locn1)]];
};

// Uniforms should have Model, Projection, _Time in exact order for shader to work,
// because they are laying in a single buffer
struct Uniforms
{
    float4x4 Model;
    float4x4 Projection;
    float4 _Time;

    float4 test_color;
};

vertex RasterizerData vertexShader(Vertex v [[stage_in]],
                                   constant Uniforms& u [[buffer(0)]])
{
    RasterizerData out;

    out.position = u.Projection * u.Model * float4(v.position, 1);
    out.uv = v.texcoord;

    return out;
}

fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                               constant Uniforms& u [[buffer(0)]],
                               texture2d<float> Texture [[texture(0)]],
                               sampler TextureSmplr [[sampler(0)]])
{
    return u.test_color * Texture.sample(TextureSmplr, in.uv);
}"#;

const FRAGMENT_WITH_ARRAY: &str = r#"#version 100
varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 test_color[10];

void main() {
    gl_FragColor = test_color[5] * texture2D(Texture, uv);
}"#;

pub const METAL_WITH_ARRAY: &str = r#"#include <metal_stdlib>
using namespace metal;

struct Vertex
{
    float3 position    [[attribute(0)]];
    float2 texcoord    [[attribute(1)]];
    float4 color0      [[attribute(2)]];
};

struct RasterizerData
{
    float4 position [[position]];
    float4 color [[user(locn0)]];
    float2 uv [[user(locn1)]];
};

// Uniforms should have Model, Projection, _Time for material shaders to work
struct Uniforms
{
    float4x4 Model;
    float4x4 Projection;
    float4 _Time;

    float4 test_color[10];
};

vertex RasterizerData vertexShader(Vertex v [[stage_in]],
                                   constant Uniforms& u [[buffer(0)]])
{
    RasterizerData out;

    out.position = u.Projection * u.Model * float4(v.position, 1);
    out.uv = v.texcoord;

    return out;
}

fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                               constant Uniforms& u [[buffer(0)]],
                               texture2d<float> Texture [[texture(0)]],
                               sampler TextureSmplr [[sampler(0)]])
{
    return u.test_color[5] * Texture.sample(TextureSmplr, in.uv);
}"#;

fn window_conf() -> Conf {
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    let apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };
    Conf {
        window_title: "Shaders".to_owned(),
        platform: conf::Platform {
            apple_gfx_api,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };

    let ctx = unsafe { get_internal_gl().quad_context };
    let mat = load_material(
        match ctx.info().backend {
            Backend::OpenGl => ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            Backend::Metal => ShaderSource::Msl { program: METAL },
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("test_color", UniformType::Float4)],
            pipeline_params,
            ..Default::default()
        },
    )
    .unwrap();

    let mat_with_array = load_material(
        match ctx.info().backend {
            Backend::OpenGl => ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT_WITH_ARRAY,
            },
            Backend::Metal => ShaderSource::Msl {
                program: METAL_WITH_ARRAY,
            },
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
