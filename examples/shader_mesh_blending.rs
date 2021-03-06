use macroquad::prelude::*;
use macroquad::models::{Vertex, Mesh};
use macroquad::color::{Color, WHITE};
use macroquad::texture::Texture2D;
use macroquad::window::miniquad::*;

use crate::{vec2, vec3};

fn window_conf() -> Conf {
    Conf {
        window_title: "custom shaders and meshes w/ blending".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

struct QuadData {
    xi: f32,
    yi: f32,
    xf: f32,
    yf: f32
}

// creates a custom mesh.
// vertex order to draw 2 triangles:
// 0 1
// 2 3
fn quad_mesh(
    qd: QuadData,
    texture: &Texture2D,
    color: &Color,
) -> Mesh {
    Mesh {
        vertices: vec![
            Vertex {
                position: vec3(qd.xi, qd.yi, 0.0),
                uv: vec2(0.0, 0.0),
                color: *color,
            },
            Vertex {
                position: vec3(qd.xf, qd.yi, 0.0),
                uv: vec2(1.0, 0.0),
                color: *color,
            },
            Vertex {
                position: vec3(qd.xi, qd.yf, 0.0),
                uv: vec2(0.0, 1.0),
                color: *color,
            },
            Vertex {
                position: vec3(qd.xf, qd.yf, 0.0),
                uv: vec2(1.0, 1.0),
                color: *color,
            },
        ],
        indices: vec![1, 0, 2, 1, 2, 3],
        texture: Some(*texture),
    }
}

fn scale_to_fit(screen_dims:(f32, f32), image_dims:(f32, f32)) -> QuadData {
    let (w, h) = screen_dims;
    let (iw, ih) = image_dims;

    let ar = w / h;
    let iar = iw / ih;

    let s: f32 = if iar > ar { w / iw } else { h / ih };

    let ix0: f32 = (w - iw * s) / 2.0;
    let iy0: f32 = (h - ih * s) / 2.0;
    let ix1: f32 = (w + iw * s) / 2.0;
    let iy1: f32 = (h + ih * s) / 2.0;

    QuadData {
        xi: ix0,
        xf: ix1,
        yi: iy0,
        yf: iy1,
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
    let texture_bg: Texture2D = load_texture("examples/rust.png").await;
    let texture_fg: Texture2D = load_texture("examples/ferris.png").await;

    let screen_dims = (screen_width(), screen_height());
    let image_bg_dims = (texture_bg.width(), texture_bg.height());
    let image_fg_dims = (texture_fg.width(), texture_fg.height());

    // these are custom meshes. generates rects scaled to fit the screen
    let mesh_bg = quad_mesh(scale_to_fit(screen_dims, image_bg_dims), &texture_bg, &WHITE);
    let mesh_fg = quad_mesh(scale_to_fit(screen_dims, image_fg_dims), &texture_fg, &WHITE);

    // our custom material is created here
    let mat = create_material();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        clear_background(BLACK);

        set_default_camera();

        // draw rust background mesh with the default material, centered
        draw_mesh(&mesh_bg);

        // draw ferris foreground mesh with our material
        gl_use_material(mat);

            // making a triangle wave between 0-1 based on time period
            let mut ratio = (get_time()*0.5) % 1.0;
            ratio = if ratio > 0.5 { 2.0 - 2.0 * ratio } else { ratio * 2.0 };

            // assign it to the material to vary it
            mat.set_uniform("Ratio", ratio as f32);

            draw_mesh(&mesh_fg);
        
        gl_use_default_material();

        next_frame().await
    }
}
