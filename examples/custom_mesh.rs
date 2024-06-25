use macroquad::prelude::*;
use macroquad::models::{Vertex, Mesh};
use macroquad::color::{Color, WHITE};
use macroquad::texture::Texture2D;
use std::f32::consts::PI;

use crate::{vec2, vec3};

fn window_conf() -> Conf {
    Conf {
        window_title: "custom mesh".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

/*                               top view (vertex 4 is closer to view)
     /|                                      0       1
    / |                                               
   /  |                                          4    
  /   | h    tan 60º = h / (s/2)                      
 /60º |                                      3       2
------+
  s/2                                                                     */
const DEG_2_RAD:f32 = PI / 180.0;

fn create_pyramid_mesh(
    side: f32,
    texture: &Texture2D,
    color: &Color,
) -> Mesh {
    let height = side * 0.5 * (DEG_2_RAD * 60.0).tan();
    let l = side * 0.5;
    Mesh {
        vertices: vec![
            Vertex { // #0
                position: vec3(-l, 0.0, l),
                uv: vec2(0.0, 0.0),
                color: *color,
            },
            Vertex { // #1
                position: vec3(l, 0.0, l),
                uv: vec2(1.0, 0.0),
                color: *color,
            },
            Vertex { // #2
                position: vec3(l, 0.0, -l),
                uv: vec2(1.0, 1.0),
                color: *color,
            },
            Vertex { // #3
                position: vec3(-l, 0.0, -l),
                uv: vec2(0.0, 1.0),
                color: *color,
            },
            Vertex { // #4
                position: vec3(0.0, height, 0.0),
                uv: vec2(0.5, 0.5),
                color: *color,
            },
        ],
        indices: vec![
            0, 4, 1,
            1, 4, 2,
            2, 4, 3,
            3, 4, 0
        ],
        texture: Some(*texture),
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let rust_tex: Texture2D = load_texture("examples/rust.png").await;
    
    let pyramid_mesh = create_pyramid_mesh(5.0, &rust_tex, &WHITE);

    let gl = unsafe { get_internal_gl().quad_gl };

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

        let t = get_time();

        // rotate the pyramid around YY
        gl.push_model_matrix(glam::Mat4::from_rotation_y(
            t.cos() as f32,
        ));

        draw_mesh(&pyramid_mesh);

        gl.pop_model_matrix();

        next_frame().await
    }
}
