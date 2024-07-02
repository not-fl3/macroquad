use dolly::prelude::*;

use macroquad::{
    input::MouseButton,
    math::{vec3, Vec3},
    quad_gl::{
        camera::{Camera, Environment, Projection},
        color,
    },
    window::next_frame,
};

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

    let mut dolly_rig: CameraRig = CameraRig::builder()
        .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-10.0))
        .with(Smooth::new_rotation(0.7))
        .with(Arm::new(Vec3::Z * 4.0))
        .build();

    let mut camera = Camera {
        environment: Environment::Skybox(skybox),
        depth_enabled: true,
        projection: Projection::Perspective,
        position: vec3(0., 1.5, 4.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        z_near: 0.1,
        z_far: 1500.0,
        ..Default::default()
    };

    // let mut canvas = ctx.new_canvas();
    // canvas.draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
    // ShapeBuilder::rectangle(vec2(100., 100.), RED)
    //     .position(vec2(100., 100.))
    //     .draw(&mut canvas);

    let mut zoom = 4.0;
    loop {
        ctx.clear_screen(color::WHITE);

        if ctx.is_mouse_button_down(MouseButton::Left) {
            dolly_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(ctx.mouse_delta().x * 100., ctx.mouse_delta().y * 100.);
        }
        if ctx.mouse_wheel().1 != 0.0 {
            zoom -= ctx.mouse_wheel().1 * 0.4;
            zoom = zoom.clamp(1.8, 10.0);
            dolly_rig.driver_mut::<Arm>().offset = (Vec3::Z * zoom).into();
        }
        let delta = 0.1;
        let dolly_transform = dolly_rig.update(delta);
        camera.position = dolly_transform.position.into();
        camera.up = dolly_transform.up();
        let p: Vec3 = dolly_transform.position.into();
        let f: Vec3 = dolly_transform.forward::<Vec3>().into();
        camera.target = p + f;

        scene.draw(&camera);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
