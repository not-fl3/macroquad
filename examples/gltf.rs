use dolly::prelude::*;
use macroquad::prelude::*;

async fn game(ctx: macroquad::Context3) {
    let helmet = ctx.load_gltf("examples/DamagedHelmet.gltf").await.unwrap();
    let _helmet = ctx.scene().add_model(helmet);

    let skybox: &[&[u8]] = &[
        &include_bytes!("skybox/skybox_px.png")[..],
        &include_bytes!("skybox/skybox_nx.png")[..],
        &include_bytes!("skybox/skybox_py.png")[..],
        &include_bytes!("skybox/skybox_ny.png")[..],
        &include_bytes!("skybox/skybox_pz.png")[..],
        &include_bytes!("skybox/skybox_nz.png")[..],
    ];
    let skybox = ctx.load_cubemap(skybox);

    let mut camera_rig: CameraRig = CameraRig::builder()
        .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-30.0))
        .with(Smooth::new_rotation(0.7))
        .with(Arm::new(Vec3::Z * 4.0))
        .build();

    let camera = ctx.scene().add_camera(Camera {
        environment: Environment::Skybox(skybox),
        depth_enabled: true,
        projection: Projection::Perspective,
        position: CameraPosition::Camera3D {
            position: vec3(0., 1.5, 4.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            fovy: 45.,
            projection: macroquad::camera::Projection::Perspective,
            aspect: None,
        },
        z_near: 0.1,
        ..Default::default()
    });

    let mut canvas = ctx.scene().fullscreen_canvas(0);
    canvas.draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
    canvas.draw_text("TEXT BELOW!!!", 400.0, 400.0, 30.0, BLUE);
    canvas.draw_rectangle(300., 200., 100., 100., RED);

    let mut canvas = ctx.scene().fullscreen_canvas(1);
    canvas.draw_rectangle(100., 350., 100., 100., GREEN);
    canvas.draw_text("TEXT ABOVE!!!", 400.0, 300.0, 30.0, YELLOW);

    let mut angles = vec2(0., 0.);
    loop {
        if is_mouse_button_down(MouseButton::Left) {
            angles += mouse_delta();
            camera_rig.driver_mut::<YawPitch>().rotate_yaw_pitch(mouse_delta().x * 100., mouse_delta().y * 100.);
        }
        let dolly_transform = camera_rig.update(get_frame_time());
        camera.update(|camera| {
            camera.position = CameraPosition::Camera3D {
                position: dolly_transform.position,
                up: dolly_transform.up(),
                target: dolly_transform.position + dolly_transform.forward(),
                fovy: 45.,
                projection: macroquad::camera::Projection::Perspective,
                aspect: None,
            }
        });

        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
