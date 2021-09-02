use macroquad::prelude::*;

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;

pub struct FirstPersonCamera {
    pub front: Vec3,
    pub position: Vec3,
    pub up: Vec3,
    pub field_of_view: f32,
    pub facing: Vec3, // Facing vector without rotation for head tilt
    pub right: Vec3,
    pub move_speed: f32,
    pub world_up: Vec3,
    pub ghost: bool, // Full flying camera (not using `self.facing`)
    pub boost: bool,

    yaw: f32,
    pitch: f32,

    pub last_mouse_position: Vec2,
    pub enabled: bool,
}

impl FirstPersonCamera {
    pub fn new() -> Self {
        Self {
            front: vec3(0.0, 0.0, 1.0),
            position: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            field_of_view: 70.0,
            ghost: false,
            world_up: vec3(0.0, 1.0, 0.0),
            right: vec3(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            facing: vec3(0.0, 0.0, 0.0),
            move_speed: 25.0,
            boost: false,

            last_mouse_position: Vec2::new(0.0, 0.0),
            enabled: true,
        }
    }

    pub fn update(&mut self, delta: f32) {
        if !self.enabled {
            return;
        }

        if is_key_down(KeyCode::Up) {
            self.position += self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Down) {
            self.position -= self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Right) {
            self.position += self.right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            self.position -= self.right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::W) {
            self.position += self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            self.position -= self.front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            self.position += self.right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            self.position -= self.right * MOVE_SPEED;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - self.last_mouse_position;
        self.last_mouse_position = mouse_position;

        self.yaw += mouse_delta.x * delta * LOOK_SPEED;
        self.pitch += mouse_delta.y * delta * -LOOK_SPEED;
        self.pitch = self.pitch.clamp(-89f32.to_radians(), 89f32.to_radians());

        self.front = vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        self.facing = vec3(self.front.x, 0.0, self.front.z).normalize();

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn get_macroquad_camera(&self) -> Camera3D {
        Camera3D {
            position: self.position,
            up: self.up,
            target: self.position + self.front,
            ..Default::default()
        }
    }
}
