use dolly::prelude::*;

use macroquad::{
    input::MouseButton,
    math::{vec3, Vec3},
    quad_gl::{
        camera::{Camera, Environment, Projection},
        color,
    },
    shapes::*,
    window::next_frame,
};

mod orbit_camera;

async fn game(ctx: macroquad::Context) {
    unsafe {
        macroquad::miniquad::gl::glEnable(macroquad::miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS)
    };

    let mut scene = ctx.new_scene();

    let helmet = ctx
        .resources
        .load_gltf("examples/DamagedHelmet.gltf")
        .await
        .unwrap();
    let _helmet = scene.add_model(&helmet);
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

    let mut zoom = 4.0;
    loop {
        ctx.clear_screen(color::WHITE);
        orbit.orbit(&ctx);
        scene.draw(&orbit.camera);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
