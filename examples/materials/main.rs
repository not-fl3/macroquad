use macroquad::{
    math::vec2,
    quad_gl::{
        camera::Environment,
        color::{self, Color},
        scene::Shader,
        ui::{hash, widgets},
    },
    window::next_frame,
};

mod orbit_camera;
mod ui;

async fn game(ctx: macroquad::Context) {
    let mut scene = ctx.new_scene();
    let mut helmet = ctx
        .resources
        .load_gltf("examples/DamagedHelmet.gltf")
        .await
        .unwrap();
    helmet.nodes[0].materials[0].shader = Shader::new(
        ctx.quad_ctx.lock().unwrap().as_mut(),
        vec![
            ("WaveThing".to_string(), miniquad::UniformType::Float1, 1),
            ("_Time".to_string(), miniquad::UniformType::Float1, 1),
        ],
        None,
        Some(VERTEX),
    );
    let helmet = scene.add_model(&helmet);
    let skybox = ctx
        .resources
        .load_cubemap(
            "examples/skybox/skybox_px.png",
            "examples/skybox/skybox_nx.png",
            "examples/skybox/skybox_py.png",
            "examples/skybox/skybox_ny.png",
            "examples/skybox/skybox_pz.png",
            "examples/skybox/skybox_nz.png",
        )
        .await
        .unwrap();
    let mut orbit = orbit_camera::OrbitCamera::new();
    orbit.camera.environment = Environment::Skybox(skybox);
    let mut canvas = ctx.new_canvas();
    let (color_picker_texture, _) = ui::color_picker_texture(&ctx, 200, 200);
    let mut wave = 0.0f32;
    let mut t = 0.0f32;
    loop {
        ctx.clear_screen(color::WHITE);
        canvas.clear();

        orbit.orbit(&ctx);

        let mat = scene.materials(&helmet).next().unwrap();
        widgets::Window::new(hash!(), vec2(400., 200.), vec2(320., 400.))
            .label("Inspector")
            .titlebar(true)
            .ui(&mut *ctx.root_ui(), |ui| {
                let mut c: Color = mat.color.into();
                ui::colorbox(&ctx, ui, hash!(), "Color", &mut c, &color_picker_texture);
                mat.color = c.into();

                ui.drag(hash!(), "WaveThing", (0.0, 100.0), &mut wave);
                mat.shader.set_uniform("WaveThing", wave * 0.01);
                mat.shader.set_uniform("_Time", t);
                t += 0.1;
                ui.drag(hash!(), "Metallic", (-1.0, 1.0), &mut mat.metallic);
                ui.drag(hash!(), "Roughness", (0.0, 1.0), &mut mat.roughness);

                if let Some(wave) = &mat.base_color_texture {
                    ui.texture(wave, 100.0, 100.0);
                }
            });

        ctx.root_ui().draw(&mut canvas);

        scene.draw(&orbit.camera);
        canvas.draw();

        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}

const VERTEX: &str = r#"
uniform float WaveThing;
uniform float _Time;

#include "common_vertex.glsl"

void vertex() {
    gl_Position.x *= 1.0 + sin(in_position.z * 10.0 + _Time) * WaveThing;
}
"#;
