use macroquad::prelude::*;

use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Label, TreeNode},
};

use macroquad::color;

enum Uniform {
    Float1(String),
    Float2(String, String),
    Float3(String, String, String),
    Color(Vec3),
}

impl Uniform {
    fn uniform_type(&self) -> UniformType {
        match self {
            Uniform::Float1(_) => UniformType::Float1,
            Uniform::Float2(_, _) => UniformType::Float2,
            Uniform::Float3(_, _, _) => UniformType::Float3,
            Uniform::Color(_) => UniformType::Float3,
        }
    }
}

fn color_picker_texture(w: usize, h: usize) -> (Texture2D, Image) {
    let ratio = 1.0 / h as f32;

    let mut image = Image::gen_image_color(w as u16, h as u16, WHITE);
    let image_data = image.get_image_data_mut();

    for j in 0..h {
        for i in 0..w {
            let lightness = 1.0 - i as f32 * ratio;
            let hue = j as f32 * ratio;

            image_data[i + j * w] = color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    (Texture2D::from_image(&image), image)
}

#[macroquad::main("Shadertoy")]
async fn main() {
    let ferris = load_texture("examples/rust.png").await.unwrap();
    let (color_picker_texture, color_picker_image) = color_picker_texture(200, 200);

    let mut fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
    let mut vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

    let pipeline_params = PipelineParams {
        depth_write: true,
        depth_test: Comparison::LessOrEqual,
        ..Default::default()
    };

    let mut material = load_material(
        ShaderSource {
            glsl_vertex: Some(&vertex_shader),
            glsl_fragment: Some(&fragment_shader),
            metal_shader: None,
        },
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
    }
    let mut mesh = Mesh::Sphere;

    let mut camera = Camera3D {
        position: vec3(-15., 15., -5.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 5., -5.),
        ..Default::default()
    };

    let mut colorpicker_window = false;
    let mut color_picking_uniform = None;

    let mut new_uniform_window = false;
    let mut new_uniform_name = String::new();
    let mut uniforms: Vec<(String, Uniform)> = vec![];

    loop {
        clear_background(WHITE);

        set_camera(&camera);

        draw_grid(
            20,
            1.,
            Color::new(0.55, 0.55, 0.55, 0.75),
            Color::new(0.75, 0.75, 0.75, 0.75),
        );

        gl_use_material(&material);
        match mesh {
            Mesh::Plane => draw_plane(vec3(0., 2., 0.), vec2(5., 5.), Some(&ferris), WHITE),
            Mesh::Sphere => draw_sphere(vec3(0., 6., 0.), 5., Some(&ferris), WHITE),
            Mesh::Cube => draw_cube(vec3(0., 5., 0.), vec3(10., 10., 10.), Some(&ferris), WHITE),
        }
        gl_use_default_material();

        set_default_camera();

        let mut need_update = false;

        widgets::Window::new(hash!(), vec2(20., 20.), vec2(470., 650.))
            .label("Shader")
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, "Camera: ");
                ui.same_line(0.0);
                if ui.button(None, "Ortho") {
                    camera.projection = Projection::Orthographics;
                }
                ui.same_line(0.0);
                if ui.button(None, "Perspective") {
                    camera.projection = Projection::Perspective;
                }
                ui.label(None, "Mesh: ");
                ui.same_line(0.0);
                if ui.button(None, "Sphere") {
                    mesh = Mesh::Sphere;
                }
                ui.same_line(0.0);
                if ui.button(None, "Cube") {
                    mesh = Mesh::Cube;
                }
                ui.same_line(0.0);
                if ui.button(None, "Plane") {
                    mesh = Mesh::Plane;
                }

                ui.label(None, "Uniforms:");
                ui.separator();

                for (i, (name, uniform)) in uniforms.iter_mut().enumerate() {
                    ui.label(None, &format!("{}", name));
                    ui.same_line(120.0);

                    match uniform {
                        Uniform::Float1(x) => {
                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(200.0, 19.0))
                                .filter_numbers()
                                .ui(ui, x);

                            if let Ok(x) = x.parse::<f32>() {
                                material.set_uniform(name, x);
                            }
                        }
                        Uniform::Float2(x, y) => {
                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(99.0, 19.0))
                                .filter_numbers()
                                .ui(ui, x);

                            ui.same_line(0.0);

                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(99.0, 19.0))
                                .filter_numbers()
                                .ui(ui, y);

                            if let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) {
                                material.set_uniform(name, (x, y));
                            }
                        }
                        Uniform::Float3(x, y, z) => {
                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(65.0, 19.0))
                                .filter_numbers()
                                .ui(ui, x);

                            ui.same_line(0.0);

                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(65.0, 19.0))
                                .filter_numbers()
                                .ui(ui, y);

                            ui.same_line(0.0);

                            widgets::InputText::new(hash!(hash!(), i))
                                .size(vec2(65.0, 19.0))
                                .filter_numbers()
                                .ui(ui, z);

                            if let (Ok(x), Ok(y), Ok(z)) =
                                (x.parse::<f32>(), y.parse::<f32>(), z.parse::<f32>())
                            {
                                material.set_uniform(name, (x, y, z));
                            }
                        }

                        Uniform::Color(color) => {
                            let mut canvas = ui.canvas();

                            let cursor = canvas.cursor();

                            canvas.rect(
                                Rect::new(cursor.x + 20.0, cursor.y, 50.0, 18.0),
                                Color::new(0.2, 0.2, 0.2, 1.0),
                                Color::new(color.x, color.y, color.z, 1.0),
                            );

                            if ui.button(None, "change") {
                                colorpicker_window = true;
                                color_picking_uniform = Some(name.to_owned());
                            }
                            material.set_uniform(name, (color.x, color.y, color.z));
                        }
                    }
                }
                ui.separator();
                if ui.button(None, "New uniform") {
                    new_uniform_window = true;
                }
                TreeNode::new(hash!(), "Fragment shader")
                    .init_unfolded()
                    .ui(ui, |ui| {
                        if ui.editbox(hash!(), vec2(440., 200.), &mut fragment_shader) {
                            need_update = true;
                        };
                    });
                ui.tree_node(hash!(), "Vertex shader", |ui| {
                    if ui.editbox(hash!(), vec2(440., 300.), &mut vertex_shader) {
                        need_update = true;
                    };
                });

                if let Some(ref error) = error {
                    Label::new(error).multiline(14.0).ui(ui);
                }
            });

        if new_uniform_window {
            widgets::Window::new(hash!(), vec2(100., 100.), vec2(200., 80.))
                .label("New uniform")
                .ui(&mut *root_ui(), |ui| {
                    if ui.active_window_focused() == false {
                        new_uniform_window = false;
                    }
                    ui.input_text(hash!(), "Name", &mut new_uniform_name);
                    let uniform_type = ui.combo_box(
                        hash!(),
                        "Type",
                        &["Float1", "Float2", "Float3", "Color"],
                        None,
                    );

                    if ui.button(None, "Add") {
                        if new_uniform_name.is_empty() == false {
                            let uniform = match uniform_type {
                                0 => Uniform::Float1("0".to_string()),
                                1 => Uniform::Float2("0".to_string(), "0".to_string()),
                                2 => Uniform::Float3(
                                    "0".to_string(),
                                    "0".to_string(),
                                    "0".to_string(),
                                ),
                                3 => Uniform::Color(vec3(0.0, 0.0, 0.0)),
                                _ => unreachable!(),
                            };
                            uniforms.push((new_uniform_name.clone(), uniform));
                            new_uniform_name.clear();
                            need_update = true;
                        }
                        new_uniform_window = false;
                    }

                    ui.same_line(0.0);
                    if ui.button(None, "Cancel") {
                        new_uniform_window = false;
                    }
                });
        }

        if colorpicker_window {
            colorpicker_window &= widgets::Window::new(hash!(), vec2(140., 100.), vec2(210., 240.))
                .label("Colorpicker")
                .ui(&mut *root_ui(), |ui| {
                    if ui.active_window_focused() == false {
                        colorpicker_window = false;
                    }

                    let mut canvas = ui.canvas();
                    let cursor = canvas.cursor();
                    let mouse = mouse_position();
                    let x = mouse.0 as i32 - cursor.x as i32;
                    let y = mouse.1 as i32 - (cursor.y as i32 + 20);

                    let color = color_picker_image
                        .get_pixel(x.max(0).min(199) as u32, y.max(0).min(199) as u32);

                    canvas.rect(
                        Rect::new(cursor.x, cursor.y, 200.0, 18.0),
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        Color::new(color.r, color.g, color.b, 1.0),
                    );
                    canvas.image(
                        Rect::new(cursor.x, cursor.y + 20.0, 200.0, 200.0),
                        &color_picker_texture,
                    );

                    if x >= 0 && x < 200 && y >= 0 && y < 200 {
                        canvas.rect(
                            Rect::new(mouse.0 - 3.5, mouse.1 - 3.5, 7.0, 7.0),
                            Color::new(0.3, 0.3, 0.3, 1.0),
                            Color::new(1.0, 1.0, 1.0, 1.0),
                        );

                        if is_mouse_button_down(MouseButton::Left) {
                            colorpicker_window = false;
                            let uniform_name = color_picking_uniform.take().unwrap();

                            uniforms
                                .iter_mut()
                                .find(|(name, _)| name == &uniform_name)
                                .unwrap()
                                .1 = Uniform::Color(vec3(color.r, color.g, color.b));
                        }
                    }
                });
        }

        if need_update {
            let uniforms = uniforms
                .iter()
                .map(|(name, uniform)| (name.clone(), uniform.uniform_type()))
                .collect::<Vec<_>>();

            match load_material(
                ShaderSource {
                    glsl_vertex: Some(&vertex_shader),
                    glsl_fragment: Some(&fragment_shader),
                    metal_shader: None,
                },
                MaterialParams {
                    pipeline_params,
                    uniforms,
                    textures: vec![],
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

        next_frame().await
    }
}

const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";
