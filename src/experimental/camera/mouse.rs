use crate::prelude::screen_width;
use crate::prelude::*;
use crate::Vec2;

/// 2D camera that can be controlled by mouse. Offset and scale can be changed.
///
/// Note: You can get a [`Camera2D`] using `let cam2d: Camera2D = (&cam).into();
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub offset: Vec2,
    pub scale: f32,

    last_mouse_pos: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec2::ZERO, 1.0)
    }
}

impl Camera {
    pub fn new(offset: Vec2, scale: f32) -> Self {
        Self {
            offset,
            scale,

            last_mouse_pos: Vec2::new(0., 0.),
        }
    }

    /// If `wheel_value` has positive value, scale cam around point `center` by factor `scale_factor`.
    /// If `wheel_value` is negative, then scale by `1.0/scale_factor`. If `wheel_value` equals `0.0` or `scale_factor` equals `1.0`, nothing happens.
    pub fn scale_wheel(&mut self, center: Vec2, wheel_value: f32, scale_factor: f32) {
        if wheel_value > 0. {
            self.scale_mul(center, scale_factor);
        } else if wheel_value < 0. {
            self.scale_mul(center, 1.0 / scale_factor);
        }
    }

    /// Adds `mul_to_scale` to current scale of cam. Scale is changed around point `center`.
    pub fn scale_mul(&mut self, center: Vec2, mul_to_scale: f32) {
        self.scale_new(center, self.scale * mul_to_scale);
    }

    /// Replace current scale of camera with `new_scale`. Scale is changed around point `center`.
    pub fn scale_new(&mut self, center: Vec2, new_scale: f32) {
        self.offset = (self.offset - center) * (new_scale / self.scale) + center;
        self.scale = new_scale;
    }

    /// Update camera position by new mouse position. This method must be run at every frame. `should_offset` controls if the camera should actually move or not.
    ///
    /// Note: It's better to use [`mouse_position_local`] with this method, otherwise if you use [`mouse_position`] the movement is way too big.
    pub fn update(&mut self, mouse_pos: Vec2, should_offset: bool) {
        if should_offset {
            self.offset += mouse_pos - self.last_mouse_pos;
        }
        self.last_mouse_pos = mouse_pos;
    }
}

impl Into<Camera2D> for &Camera {
    fn into(self) -> Camera2D {
        let aspect = screen_width() / screen_height();
        Camera2D {
            zoom: vec2(self.scale, -self.scale * aspect),
            offset: vec2(self.offset.x, -self.offset.y),
            target: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
            viewport: None,
        }
    }
}
