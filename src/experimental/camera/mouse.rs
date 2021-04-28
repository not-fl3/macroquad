use crate::prelude::screen_width;
use crate::prelude::*;
use crate::Vec2;

/// 2D camera that can be controlled by mouse. Offset and scale can be changed.
#[derive(Debug, Clone)]
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

    /// If `wheel_value` has positive value, scale cam around point `mouse_pos` by factor `scale_factor`. If `wheel_value` is negative, then scale by `1.0/scale_factor`. If `mouse_pos` equals `0`, does nothing.
    pub fn scale_wheel(&mut self, mouse_pos: Vec2, wheel_value: f32, scale_factor: f32) {
        if wheel_value > 0. {
            self.scale_mul(mouse_pos, scale_factor);
        } else if wheel_value < 0. {
            self.scale_mul(mouse_pos, 1.0 / scale_factor);
        }
    }

    /// Adds `mul_to_scale` to current scale of cam. Scale is changed around point `mouse_pos`.
    pub fn scale_mul(&mut self, mouse_pos: Vec2, mul_to_scale: f32) {
        self.scale_new(mouse_pos, self.scale * mul_to_scale);
    }

    /// Replace current scale of mouse by new value. Scale is changed around point `mouse_pos`.
    pub fn scale_new(&mut self, mouse_pos: Vec2, new_scale: f32) {
        self.offset = (self.offset - mouse_pos) * (new_scale / self.scale) + mouse_pos;
        self.scale = new_scale;
    }

    /// Update camera position by new mouse position. This method must be run at every frame in case to remember previous mouse position.
    pub fn update(&mut self, mouse_pos: Vec2, should_offset: bool) {
        if should_offset {
            self.offset += mouse_pos - self.last_mouse_pos;
        }
        self.last_mouse_pos = mouse_pos;
    }
}

impl Into<Camera2D> for Camera {
    fn into(self) -> Camera2D {
        let aspect = screen_width() / screen_height();
        Camera2D {
            zoom: vec2(self.scale, -self.scale * aspect),
            offset: vec2(self.offset.x, -self.offset.y),
            target: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
        }
    }
}
