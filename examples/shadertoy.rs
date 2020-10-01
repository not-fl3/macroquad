use macroquad::*;

use macroquad::megaui::widgets::{Label, TreeNode};

use glam::vec3;

#[macroquad::main("Shadertoy")]
async fn main() {
    let ferris = load_texture("examples/rust.png").await;

    let mut fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
    let mut vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

    let pipeline_params = PipelineParams {
        depth_write: true,
        depth_test: Comparison::LessOrEqual,
        ..Default::default()
    };
    let mut material = load_material(
        &vertex_shader,
        &fragment_shader,
        MaterialParams {
            pipeline_params,
            ..Default::default()
        },
    )
    .unwrap();
    let mut error: Option<String> = None;

    enum Mesh {
        Sphere,
        Cube,
        Plane,
    };
    let mut mesh = Mesh::Sphere;

    let mut camera = Camera3D {
        position: vec3(-15., 15., -5.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 5., -5.),
        ..Default::default()
    };

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }

        clear_background(WHITE);

        set_camera(camera);

        draw_grid(20, 1.);

        gl_use_material(material);
        match mesh {
            Mesh::Plane => draw_plane(vec3(0., 2., 0.), vec2(5., 5.), ferris, WHITE),
            Mesh::Sphere => draw_sphere(vec3(0., 6., 0.), 5., ferris, WHITE),
            Mesh::Cube => draw_cube(vec3(0., 5., 0.), vec3(10., 10., 10.), ferris, WHITE),
        }
        gl_use_default_material();

        // Back to screen space, render some text

        set_default_camera();
        draw_window(
            hash!(),
            vec2(20., 20.),
            vec2(450., 650.),
            WindowParams {
                label: "Shader".to_string(),
                close_button: false,
                ..Default::default()
            },
            |ui| {
                let mut need_update = false;

                ui.label(None, "Camera: ");
                ui.same_line();
                if ui.button(None, "Ortho") {
                    camera.projection = Projection::Orthographics;
                }
                ui.same_line();
                if ui.button(None, "Perspective") {
                    camera.projection = Projection::Perspective;
                }
                ui.label(None, "Mesh: ");
                ui.same_line();
                if ui.button(None, "Sphere") {
                    mesh = Mesh::Sphere;
                }
                ui.same_line();
                if ui.button(None, "Cube") {
                    mesh = Mesh::Cube;
                }
                ui.same_line();
                if ui.button(None, "Plane") {
                    mesh = Mesh::Plane;
                }

                TreeNode::new(hash!(), "Fragment shader")
                    .init_unfolded()
                    .ui(ui, |ui| {
                        if ui.editbox(
                            hash!(),
                            megaui::Vector2::new(440., 200.),
                            &mut fragment_shader,
                        ) {
                            need_update = true;
                        };
                    });
                ui.tree_node(hash!(), "Vertex shader", |ui| {
                    if ui.editbox(
                        hash!(),
                        megaui::Vector2::new(440., 300.),
                        &mut vertex_shader,
                    ) {
                        need_update = true;
                    };
                });

                if ui.button(None, "Update") || need_update {
                    match load_material(
                        &vertex_shader,
                        &fragment_shader,
                        MaterialParams {
                            pipeline_params,
                            ..Default::default()
                        },
                    ) {
                        Ok(new_material) => {
                            material = new_material;
                            error = None;
                        }
                        Err(err) => {
                            error = Some(format!("{:#?}", err));
                        }
                    }
                }
                if let Some(ref error) = error {
                    Label::new(error).multiline(14.0).ui(ui);
                }
            },
        );

        next_frame().await
    }
}

const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
    
uniform sampler2D Texture;

void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
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
}
";
