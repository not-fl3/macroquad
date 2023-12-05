//! 2D and 3D camera.

use crate::{
    get_context,
    math::Rect,
    texture::RenderTarget,
    window::{screen_height, screen_width},
};
use glam::{vec2, vec3, Mat4, Vec2, Vec3};

pub trait Camera {
    fn matrix(&self) -> Mat4;
    fn depth_enabled(&self) -> bool;
    fn render_pass(&self) -> Option<miniquad::RenderPass>;
    fn viewport(&self) -> Option<(i32, i32, i32, i32)>;
}

#[derive(Debug)]
pub struct Camera2D {
    /// Rotation in degrees.
    pub rotation: f32,
    /// Scaling, should be (1.0, 1.0) by default.
    pub zoom: Vec2,
    /// Rotation and zoom origin.
    pub target: Vec2,
    /// Displacement from target.
    pub offset: Vec2,

    /// If "render_target" is set - camera will render to texture.
    ///
    /// Otherwise to the screen.
    pub render_target: Option<RenderTarget>,

    /// Part of the screen to render to.
    ///
    /// None means the whole screen.
    ///
    /// Viewport do not affect camera space, just the render position on the screen.
    ///
    /// Useful for things like splitscreen.
    pub viewport: Option<(i32, i32, i32, i32)>,
}

impl Camera2D {
    /// Will make camera space equals given rect.
    pub fn from_display_rect(rect: Rect) -> Camera2D {
        let target = vec2(rect.x + rect.w / 2., rect.y + rect.h / 2.);

        Camera2D {
            target,
            zoom: vec2(1. / rect.w * 2., -1. / rect.h * 2.),
            offset: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
            viewport: None,
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
            viewport: None,
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
        let mat_origin = Mat4::from_translation(vec3(-self.target.x, -self.target.y, 0.0));
        let mat_rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let invert_y = if self.render_target.is_some() {
            1.0
        } else {
            -1.0
        };
        let mat_scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y * invert_y, 1.0));
        let mat_translation = Mat4::from_translation(vec3(self.offset.x, self.offset.y, 0.0));

        mat_translation * ((mat_scale * mat_rotation) * mat_origin)
    }

    fn depth_enabled(&self) -> bool {
        false
    }

    fn render_pass(&self) -> Option<miniquad::RenderPass> {
        self.render_target.as_ref().map(|rt| rt.render_pass)
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        self.viewport
    }
}

impl Camera2D {
    /// Returns the screen space position for a 2d camera world space position.
    ///
    /// Screen position in window space - from (0, 0) to (screen_width, screen_height()).
    pub fn world_to_screen(&self, point: Vec2) -> Vec2 {
        let mat = self.matrix();
        let transform = mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(
            (transform.x / 2. + 0.5) * screen_width(),
            (0.5 - transform.y / 2.) * screen_height(),
        )
    }

    /// Returns the world space position for a 2d camera screen space position.
    ///
    /// Point is a screen space position, often mouse x and y.
    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        let point = vec2(
            point.x / screen_width() * 2. - 1.,
            1. - point.y / screen_height() * 2.,
        );
        let inv_mat = self.matrix().inverse();
        let transform = inv_mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(transform.x, transform.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective,
    Orthographics,
}

#[derive(Debug)]
pub struct Camera3D {
    /// Camera position.
    pub position: Vec3,
    /// Camera target it looks-at.
    pub target: Vec3,
    /// Camera up vector (rotation over its axis).
    pub up: Vec3,
    /// Camera field-of-view aperture in Y (degrees)
    /// in perspective, used as near plane width in orthographic.
    pub fovy: f32,
    /// Screen aspect ratio.
    ///
    /// By default aspect is calculated with screen_width() / screen_height() on each frame.
    pub aspect: Option<f32>,
    /// Camera projection type, perspective or orthographics.
    pub projection: Projection,

    /// If "render_target" is set - camera will render to texture.
    ///
    /// Otherwise to the screen.
    pub render_target: Option<RenderTarget>,

    /// Part of the screen to render to.
    ///
    /// None means the whole screen.
    ///
    /// Viewport do not affect camera space, just the render position on the screen.
    ///
    /// Useful for things like splitscreen.
    pub viewport: Option<(i32, i32, i32, i32)>,
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
            render_target: None,
            viewport: None,
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
        self.render_target.as_ref().map(|rt| rt.render_pass)
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        self.viewport
    }
}

/// Set active 2D or 3D camera.
pub fn set_camera(camera: &dyn Camera) {
    let context = get_context();

    // flush previous camera draw calls
    context.perform_render_passes();

    context.gl.render_pass(camera.render_pass());
    context.gl.viewport(camera.viewport());
    context.gl.depth_test(camera.depth_enabled());
    context.camera_matrix = Some(camera.matrix());
}

/// Reset default 2D camera mode.
pub fn set_default_camera() {
    let context = get_context();

    // flush previous camera draw calls
    context.perform_render_passes();

    context.gl.render_pass(None);
    context.gl.depth_test(false);
    context.camera_matrix = None;
}

pub(crate) struct CameraState {
    render_pass: Option<miniquad::RenderPass>,
    depth_test: bool,
    matrix: Option<Mat4>,
}

pub fn push_camera_state() {
    let context = get_context();

    let camera_state = CameraState {
        render_pass: context.gl.get_active_render_pass(),
        depth_test: context.gl.is_depth_test_enabled(),
        matrix: context.camera_matrix,
    };
    context.camera_stack.push(camera_state);
}

pub fn pop_camera_state() {
    let context = get_context();

    if let Some(camera_state) = context.camera_stack.pop() {
        context.perform_render_passes();

        context.gl.render_pass(camera_state.render_pass);
        context.gl.depth_test(camera_state.depth_test);
        context.camera_matrix = camera_state.matrix;
    }
}
