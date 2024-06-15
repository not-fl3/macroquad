use macroquad::prelude::*;

async fn game(ctx: macroquad::Context3) {
    unsafe {
        macroquad::miniquad::gl::glEnable(macroquad::miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS)
    };

    let mut scene = ctx.new_scene();

    let helmet = ctx.load_gltf("examples/DamagedHelmet.gltf").await.unwrap();
    let _helmet = scene.add_model(&helmet);

    let skybox = ctx
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

    let mut camera = Camera {
        environment: Environment::Skybox(skybox),
        depth_enabled: true,
        projection: Projection::Perspective,
        position: vec3(0., 1.5, 4.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        z_near: 0.1,
        z_far: 15.0,
        ..Default::default()
    };

    // let mut canvas = ctx.new_canvas();
    // canvas.draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
    // ShapeBuilder::rectangle(vec2(100., 100.), RED)
    //     .position(vec2(100., 100.))
    //     .draw(&mut canvas);

    // let mut zoom = 4.0;
    loop {
        // if ctx.is_mouse_button_down(MouseButton::Left) {
        //     dolly_rig
        //         .driver_mut::<YawPitch>()
        //         .rotate_yaw_pitch(ctx.mouse_delta().x * 100., ctx.mouse_delta().y * 100.);
        // }
        // if ctx.mouse_wheel().1 != 0.0 {
        //     zoom -= ctx.mouse_wheel().1 * 0.4;
        //     zoom = zoom.clamp(1.8, 10.0);
        //     dolly_rig.driver_mut::<Arm>().offset = (Vec3::Z * zoom).into();
        // }
        // let delta = 0.1;
        // let dolly_transform = dolly_rig.update(delta);
        // camera.position = dolly_transform.position.into();
        // camera.up = dolly_transform.up();
        // let p: Vec3 = dolly_transform.position.into();
        // let f: Vec3 = dolly_transform.forward::<Vec3>().into();
        // camera.target = p + f;

        scene.draw(&camera);
        //canvas.draw();
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
