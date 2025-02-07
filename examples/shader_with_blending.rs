use macroquad::prelude::*;
use macroquad::color::WHITE;
use macroquad::texture::Texture2D;
use macroquad::window::miniquad::*;

use crate::{vec2, vec3};

fn window_conf() -> Conf {
    Conf {
        window_title: "custom shader with blending".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

const VERTEX_SHADER: &'static str = "#version 100
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
}";

// Texture is passed by macroquad automatically as this mesh has a texture assigned
// Ratio is parameterized via the material's uniform parameter
const FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
uniform float Ratio;
void main() {
    vec3 color = texture2D(Texture, uv).rgb * color.rgb;
    gl_FragColor = vec4(color, Ratio);
}";

pub fn create_material() -> Material {
    let mat = load_material(
        &VERTEX_SHADER.to_string(),
        &FRAGMENT_SHADER.to_string(),

        // this was needed because macroquad doesn't have alpha blending set by default
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
                ),
                ..Default::default()
            },
            uniforms: vec![
                ("Ratio".to_owned(), UniformType::Float1)
            ],
            textures: vec![
                //"Texture".to_owned() // this one is defined by Macroquad. assign other manually if needed.
            ],
            ..Default::default()
        },
    )
    .unwrap();

    mat.set_uniform("Ratio", 1.0 as f32); // optional. setting default parameter value

    mat
}

#[macroquad::main(window_conf())]
async fn main() {
    let rust_tex: Texture2D = load_texture("examples/rust.png").await;
    let ferris_tex: Texture2D = load_texture("examples/ferris.png").await;
    let rust_ar = rust_tex.width() / rust_tex.height();
    let ferris_ar = ferris_tex.width() / ferris_tex.height();

    // our custom material is created here
    let mat = create_material();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        clear_background(GRAY);

        set_camera(Camera3D {
            position: vec3(0., 12., 15.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            ..Default::default()
        });

        // draw rust at (0, 0, 0)
        draw_plane(vec3(0., 0., 0.), vec2(rust_ar * 5., 5.), rust_tex, WHITE);

        // draw ferris foreground mesh with our material
        gl_use_material(mat);

            // making a triangle wave between [0-1] based on time
            let mut ratio = (get_time()*0.5) % 1.0;
            ratio = if ratio > 0.5 { 2.0 - 2.0 * ratio } else { ratio * 2.0 };

            // assign it to the material to vary it
            mat.set_uniform("Ratio", ratio as f32);
            
            // draw ferris with the translucent material slightly above it
            draw_plane(vec3(0., 2., 0.), vec2(ferris_ar * 5., 5.), ferris_tex, WHITE);
        gl_use_default_material();

        next_frame().await
    }
}
