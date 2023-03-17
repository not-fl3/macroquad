use macroquad::prelude::*;

#[macroquad::main("Texture")]
async fn main() {
    let texture: Texture2D = load_texture("examples/chess.png").await.unwrap();

    let lens_material = load_material(
        ShaderSource {
            glsl_vertex: Some(LENS_VERTEX_SHADER),
            glsl_fragment: Some(LENS_FRAGMENT_SHADER),
            metal_shader: None,
        },
        MaterialParams {
            uniforms: vec![("Center".to_owned(), UniformType::Float2)],
            ..Default::default()
        },
    )
    .unwrap();

    loop {
        clear_background(WHITE);
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        let lens_center = mouse_position();

        lens_material.set_uniform("Center", lens_center);

        gl_use_material(&lens_material);
        draw_circle(lens_center.0, lens_center.1, 250.0, RED);
        gl_use_default_material();

        next_frame().await
    }
}

const LENS_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;

varying vec2 uv;
varying vec2 uv_screen;
varying vec2 center;

uniform sampler2D _ScreenTexture;

void main() {
    float gradient = length(uv);
    vec2 uv_zoom = (uv_screen - center) * gradient + center;

    gl_FragColor = texture2D(_ScreenTexture, uv_zoom);
}
"#;

const LENS_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 center;
varying lowp vec2 uv;
varying lowp vec2 uv_screen;

uniform mat4 Model;
uniform mat4 Projection;

uniform vec2 Center;

void main() {
    vec4 res = Projection * Model * vec4(position, 1);
    vec4 c = Projection * Model * vec4(Center, 0, 1);

    uv_screen = res.xy / 2.0 + vec2(0.5, 0.5);
    center = c.xy / 2.0 + vec2(0.5, 0.5);
    uv = texcoord;

    gl_Position = res;
}
";
