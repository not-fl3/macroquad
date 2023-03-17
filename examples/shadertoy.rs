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

fn load_material(
    shader_src: &str,
    uniforms: Vec<(String, macroquad::prelude::UniformType)>,
    glsl_dbg: &mut String,
    metal_dbg: &mut String,
) -> Result<Material, String> {
    use nanoshredder::ShaderTy;

    let pipeline_params = PipelineParams {
        depth_write: true,
        depth_test: Comparison::LessOrEqual,
        ..Default::default()
    };

    let mut shader = nanoshredder::Shader::new(&shader_src).map_err(|err| format!("{:?}", err))?;

    shader.add_attribute("position", ShaderTy::Vec3);
    shader.add_attribute("texcoord", ShaderTy::Vec2);

    shader.add_uniform("Projection", ShaderTy::Mat4);
    shader.add_uniform("Model", ShaderTy::Mat4);

    for (uniform_name, uniform_ty) in &uniforms {
        let ty = match uniform_ty {
            UniformType::Float1 => ShaderTy::Float,
            UniformType::Float2 => ShaderTy::Vec2,
            UniformType::Float3 => ShaderTy::Vec3,
            UniformType::Float4 => ShaderTy::Vec4,
            _ => unimplemented!(),
        };
        shader.add_uniform(&uniform_name, ty);
    }
    shader.compile().map_err(|err| format!("{:?}", err))?;

    let (vertex, fragment) = shader.generate_glsl();
    let mut metal = std::panic::catch_unwind(move || shader.generate_metal())
        .unwrap_or("generate_metal paniced".to_string());
    *glsl_dbg = format!("VERTEX\n{}\nFRAGMENT\n{}\n", vertex, fragment)
        .lines()
        .filter(|l| l.len() != 0)
        .map(|l| format!("{}\n", l))
        .collect::<String>();
    *metal_dbg = format!("{}", metal);

    let len = dbg!(32 + uniforms.iter().map(|u| u.1.size() / 4).sum::<usize>());
    let meta = miniquad::ShaderMeta {
        images: vec!["Texture".to_string()],
        uniforms: miniquad::UniformBlockLayout {
            uniforms: vec![
                miniquad::UniformDesc::new("pass_table", miniquad::UniformType::Float1).array(len),
            ],
        },
    };

    load_material2(
        ShaderSource {
            glsl_vertex: Some(&vertex),
            glsl_fragment: Some(&fragment),
            metal_shader: None,
        },
        MaterialParams {
            pipeline_params,
            uniforms,
            textures: vec![],
        },
        meta,
    )
    .map_err(|err| format!("{:?}", err))
}

#[macroquad::main("Shadertoy")]
async fn main() {
    //let ferris = load_texture("examples/rust.png").await.unwrap();
    let (color_picker_texture, color_picker_image) = color_picker_texture(256, 256);

    let mut shader_src = SHADER.to_owned();
    let mut glsl_dbg = String::new();
    let mut metal_dbg = String::new();
    let mut material = load_material(&shader_src, vec![], &mut glsl_dbg, &mut metal_dbg).unwrap();
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

        // draw_grid(
        //     20,
        //     1.,
        //     Color::new(0.55, 0.55, 0.55, 0.75),
        //     Color::new(0.75, 0.75, 0.75, 0.75),
        // );

        gl_use_material(material);
        match mesh {
            Mesh::Plane => draw_plane(vec3(0., 2., 0.), vec2(5., 5.), color_picker_texture, WHITE),
            Mesh::Sphere => draw_sphere(vec3(0., 6., 0.), 5., color_picker_texture, WHITE),
            Mesh::Cube => draw_cube(vec3(0., 5., 0.), vec3(10., 10., 10.), color_picker_texture, WHITE),
        }
        gl_use_default_material();

        set_default_camera();

        let mut need_update = false;

        widgets::Window::new(hash!(), vec2(490., 20.), vec2(450., 640.))
            .label("dbg")
            .ui(&mut *root_ui(), |ui| {
                if ui.editbox(hash!(), vec2(440., 300.), &mut glsl_dbg) {};
                ui.separator();
                ui.separator();
                if ui.editbox(hash!(), vec2(440., 300.), &mut metal_dbg) {};
            });

        widgets::Window::new(hash!(), vec2(20., 20.), vec2(455., 650.))
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

                if ui.editbox(
                    hash!(),
                    vec2(440., 460. - uniforms.len() as f32 * 22.),
                    &mut shader_src,
                ) {
                    need_update = true;
                };

                if let Some(ref error) = error {
                    let mut n = 0;
                    let mut rest = error.to_owned();
                    while !rest.is_empty() {
                        let (a, b) = rest.split_at(60.min(rest.len()));
                        Label::new(a).ui(ui);
                        rest = b.to_string();
                    }
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
                        color_picker_texture,
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

            match load_material(&shader_src, uniforms, &mut glsl_dbg, &mut metal_dbg) {
                Ok(new_material) => {
                    material.delete();

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

const SHADER: &'static str = "
varying uv: vec2
        
fn vertex(self) -> vec4 {
    self.uv = self.texcoord;
    return self.Projection
        * self.Model
        * vec4(self.position, 1);
}
        
fn pixel(self) -> vec4 {
    return #f0f
}
";
