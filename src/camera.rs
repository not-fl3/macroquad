use glam::{vec2, vec3, Mat4, Vec2};

#[derive(Clone, Copy)]
pub struct Camera2D {
    /// Rotation in degrees
    pub rotation: f32,
    /// Scaling, should be (1.0, 1.0) by default
    pub zoom: Vec2,
    /// Rotation and zoom origin
    pub target: Vec2,
    /// Displacement from target
    pub offset: Vec2,
}

impl Default for Camera2D {
    fn default() -> Camera2D {
        Camera2D {
            zoom: vec2(1., 1.),
            offset: vec2(0., 0.),
            target: vec2(0., 0.),
            rotation: 0.,
        }
    }
}

impl Camera2D {
    pub fn matrix(&self) -> Mat4 {
        // gleaned from https://github.com/raysan5/raylib/blob/master/src/core.c#L1528

        // The camera in world-space is set by
        //   1. Move it to target
        //   2. Rotate by -rotation and scale by (1/zoom)
        //      When setting higher scale, it's more intuitive for the world to become bigger (= camera become smaller),
        //      not for the camera getting bigger, hence the invert. Same deal with rotation.
        //   3. Move it by (-offset);
        //      Offset defines target transform relative to screen, but since we're effectively "moving" screen (camera)
        //      we need to do it into opposite direction (inverse transform)

        // Having camera transform in world-space, inverse of it gives the modelview transform.
        // Since (A*B*C)' = C'*B'*A', the modelview is
        //   1. Move to offset
        //   2. Rotate and Scale
        //   3. Move by -target
        let mat_origin = Mat4::from_translation(vec3(-self.target.x(), -self.target.y(), 0.0));
        let mat_rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let mat_scale = Mat4::from_scale(vec3(self.zoom.x(), self.zoom.y(), 1.0));
        let mat_translation = Mat4::from_translation(vec3(self.offset.x(), self.offset.y(), 0.0));

        mat_translation * ((mat_rotation * mat_scale) * mat_origin)
    }

    /// Returns the screen space position for a 2d camera world space position
    pub fn world_to_screen(&self, point: Vec2) -> Vec2 {
        let mat = self.matrix();
        let transform = mat.transform_point3(vec3(point.x(), point.y(), 0.));

        vec2(transform.x(), transform.y())
    }

    // Returns the world space position for a 2d camera screen space position
    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        let inv_mat = self.matrix().inverse();
        let transform = inv_mat.transform_point3(vec3(point.x(), point.y(), 0.));

        vec2(transform.x(), transform.y())
    }
}
