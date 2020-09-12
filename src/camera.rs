//! Module for conveting 2D and 3D camera descriptions into matrices
//! and some helpers like WorkdToCamera/CameraToWorld etc

use crate::{screen_height, screen_width, Rect, RenderTarget};
use glam::{vec2, vec3, Mat4, Vec2, Vec3};

pub trait Camera {
    fn matrix(&self) -> Mat4;
    fn depth_enabled(&self) -> bool;
    fn render_pass(&self) -> Option<miniquad::RenderPass>;
}

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

    /// If "render_target" is set - camera will render to texture
    /// otherwise to the screen
    pub render_target: Option<RenderTarget>,
}

impl Camera2D {
    /// Will make camera space equals given rect
    pub fn from_display_rect(rect: Rect) -> Camera2D {
        let target = vec2(rect.x + rect.w / 2., rect.y + rect.h / 2.);

        Camera2D {
            target,
            zoom: vec2(1. / rect.w * 2., 1. / rect.h * 2.),
            offset: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
        }
    }
}

impl Default for Camera2D {
    fn default() -> Camera2D {
        Camera2D {
            zoom: vec2(1., 1.),
            offset: vec2(0., 0.),
            target: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
        }
    }
}

impl Camera for Camera2D {
    fn matrix(&self) -> Mat4 {
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

    fn depth_enabled(&self) -> bool {
        false
    }

    fn render_pass(&self) -> Option<miniquad::RenderPass> {
        self.render_target.map(|rt| rt.render_pass)
    }
}

impl Camera2D {
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

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective,
    Orthographics,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera3D {
    /// Camera position
    pub position: Vec3,
    /// Camera target it looks-at
    pub target: Vec3,
    /// Camera up vector (rotation over its axis)
    pub up: Vec3,
    /// Camera field-of-view apperture in Y (degrees)
    /// in perspective, used as near plane width in orthographic
    pub fovy: f32,
    /// Screen aspect ratio
    /// By default aspect is calculated with screen_width() / screen_height() on each frame
    pub aspect: Option<f32>,
    /// Camera projection type, perspective or orthographics
    pub projection: Projection,
}

impl Default for Camera3D {
    fn default() -> Camera3D {
        Camera3D {
            position: vec3(0., -10., 0.),
            target: vec3(0., 0., 0.),
            aspect: None,
            up: vec3(0., 0., 1.),
            fovy: 45.,
            projection: Projection::Perspective,
        }
    }
}

impl Camera3D {
    const Z_NEAR: f32 = 0.01;
    const Z_FAR: f32 = 10000.0;
}
impl Camera for Camera3D {
    fn matrix(&self) -> Mat4 {
        let aspect = self.aspect.unwrap_or(screen_width() / screen_height());

        match self.projection {
            Projection::Perspective => {
                Mat4::perspective_rh_gl(self.fovy, aspect, Self::Z_NEAR, Self::Z_FAR)
                    * Mat4::look_at_rh(self.position, self.target, self.up)
            }
            Projection::Orthographics => {
                let top = self.fovy / 2.0;
                let right = top * aspect;

                Mat4::orthographic_rh_gl(-right, right, -top, top, Self::Z_NEAR, Self::Z_FAR)
                    * Mat4::look_at_rh(self.position, self.target, self.up)
            }
        }
    }

    fn depth_enabled(&self) -> bool {
        true
    }

    fn render_pass(&self) -> Option<miniquad::RenderPass> {
        unimplemented!()
    }
}
