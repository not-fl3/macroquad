use dolly::prelude::*;
use macroquad::{
    input::MouseButton,
    math::{vec3, Vec3},
    quad_gl::{
        camera::{Camera, Environment, Projection},
        color,
    },
};

pub struct OrbitCamera {
    dolly_rig: CameraRig,
    pub camera: Camera,
    zoom: f32,
}

impl OrbitCamera {
    pub fn new() -> OrbitCamera {
        let dolly_rig: CameraRig = CameraRig::builder()
            .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-10.0))
            .with(Smooth::new_rotation(0.7))
            .with(Arm::new(Vec3::Z * 4.0))
            .build();

        let camera = Camera {
            environment: Environment::SolidColor(color::BLACK),
            depth_enabled: true,
            projection: Projection::Perspective,
            position: vec3(0., 1.5, 4.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            z_near: 0.1,
            z_far: 150.0,
            ..Default::default()
        };

        OrbitCamera {
            dolly_rig,
            camera,
            zoom: 4.0,
        }
    }

    pub fn orbit(&mut self, ctx: &macroquad::Context) {
        if !ctx.root_ui().is_mouse_over(ctx.mouse_position())
            && ctx.is_mouse_button_down(MouseButton::Left)
        {
            self.dolly_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(ctx.mouse_delta().x * 100., ctx.mouse_delta().y * 100.);
        }
        if ctx.mouse_wheel().1 != 0.0 {
            self.zoom -= ctx.mouse_wheel().1 * 0.4;
            self.zoom = self.zoom.clamp(1.8, 10.0);
            self.dolly_rig.driver_mut::<Arm>().offset = (Vec3::Z * self.zoom).into();
        }
        let delta = 0.1;
        let dolly_transform = self.dolly_rig.update(delta);
        self.camera.position = dolly_transform.position.into();
        self.camera.up = dolly_transform.up();
        let p: Vec3 = dolly_transform.position.into();
        let f: Vec3 = dolly_transform.forward::<Vec3>().into();
        self.camera.target = p + f;
    }
}
