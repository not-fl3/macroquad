//! 2D and 3D camera.

use crate::{
    color::Color,
    cubemap::Cubemap,
    material::Material,
    texture::RenderTarget,
    window::{screen_height, screen_width},
};
use glam::{vec2, vec3, Mat4, Vec2, Vec3};

#[derive(Debug)]
pub enum Environment {
    Solid(Color),
    Skybox(Cubemap),
}

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective,
    Orthographics,
}

#[derive(Debug)]
pub struct Camera {
    pub depth_enabled: bool,
    pub render_target: Option<RenderTarget>,

    pub projection: Projection,
    /// Camera's point of view in world space
    pub position: CameraPosition,

    /// Rectangle on the screen where this camera's output is drawn
    /// Numbers are pixels in window-spae, x, y, width, height
    pub viewport: Option<(i32, i32, i32, i32)>,

    pub environment: Environment,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Debug, Clone)]
pub enum CameraPosition {
    Camera2D {
        /// Rotation in degrees
        rotation: f32,
        /// Scaling, (1.0, 1.0) by default
        zoom: Vec2,
        /// Rotation and zoom origin
        target: Vec2,
        /// Displacement from target
        offset: Vec2,
    },
    Camera3D {
        /// Camera position
        position: Vec3,
        /// Camera target it looks-at
        target: Vec3,
        /// Camera up vector (rotation over its axis)
        up: Vec3,
        /// Camera field-of-view aperture in Y (degrees)
        /// in perspective, used as near plane width in orthographic
        fovy: f32,
        /// Camera projection type, perspective or orthographics
        projection: Projection,

        aspect: Option<f32>,
    },
}

impl Camera {
    pub fn proj_view(&self) -> (Mat4, Mat4) {
        match &self.position {
            CameraPosition::Camera2D {
                target,
                rotation,
                zoom,
                offset,
            } => {
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
                let mat_origin = Mat4::from_translation(vec3(-target.x, -target.y, 0.0));
                let mat_rotation =
                    Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), rotation.to_radians());
                let mat_scale = Mat4::from_scale(vec3(zoom.x, zoom.y, 1.0));
                let mat_translation = Mat4::from_translation(vec3(offset.x, offset.y, 0.0));

                (
                    Mat4::IDENTITY,
                    mat_translation * ((mat_scale * mat_rotation) * mat_origin),
                )
            }
            CameraPosition::Camera3D {
                fovy,
                position,
                target,
                up,
                projection,
                aspect,
            } => {
                let aspect = aspect.unwrap_or(screen_width() / screen_height());
                match projection {
                    Projection::Perspective => (
                        Mat4::perspective_rh_gl(*fovy, aspect, self.z_near, self.z_far),
                        Mat4::look_at_rh(*position, *target, *up),
                    ),
                    Projection::Orthographics => {
                        let top = fovy / 2.0;
                        let right = top * aspect;

                        (
                            Mat4::orthographic_rh_gl(
                                -right,
                                right,
                                -top,
                                top,
                                self.z_near,
                                self.z_far,
                            ),
                            Mat4::look_at_rh(*position, *target, *up),
                        )
                    }
                }
            }
        }
    }
}

//     pub fn fixed_height(height: f32) -> Camera {
//         let aspect = screen_width() / screen_height();
//         let width = height * aspect;
//         Camera::Camera2D {
//             rotation: 0.,
//             zoom: vec2(1. / width, 1. / height),
//             target: vec2(0., 0.),
//             offset: vec2(0., 0.),
//         }
//     }

//     pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
//         let point = vec2(
//             point.x / screen_width() * 2. - 1.,
//             1. - point.y / screen_height() * 2.,
//         );
//         let inv_mat = self.matrix().inverse();
//         let transform = inv_mat.transform_point3(vec3(point.x, point.y, 0.));

//         vec2(transform.x, transform.y)
//     }
// }

// #[derive(Clone, Debug)]
// pub struct RenderState {
//     pub depth_enabled: bool,
//     pub render_target: Option<RenderTarget>,

//     ///
//     pub camera: Camera,
//     /// Rectangle on the screen where this camera's output is drawn
//     /// Numbers are pixels in window-spae, x, y, width, height
//     pub viewport: Option<(i32, i32, i32, i32)>,

//     pub material: Option<Material>,
// }

impl Default for Camera {
    fn default() -> Self {
        Camera {
            depth_enabled: false,
            render_target: None,

            projection: Projection::Orthographics,
            position: CameraPosition::Camera2D {
                target: vec2(0., 0.),
                zoom: vec2(1., 1.),
                offset: vec2(0., 0.),
                rotation: 0.,
            },
            environment: Environment::Solid(Color::new(0.2, 0.2, 0.5, 1.0)),
            viewport: None,
            z_far: 1000.,
            z_near: 3.0,
        }
    }
}

// impl RenderState {
//     pub fn matrix(&self) -> Mat4 {
//         self.camera.matrix()
//     }
// }

// /// Set active 2D or 3D camera
// pub fn set_camera(camera: &Camera) {
//     let context = get_context();

//     // flush previous camera draw calls
//     context.perform_render_passes();

//     context
//         .gl
//         .render_pass(camera.render_target.map(|rt| rt.render_pass));
//     context.gl.viewport(camera.viewport);
//     context.gl.depth_test(camera.depth_enabled);
//     context.camera_matrix = Some(camera.matrix());
// }

// /// Reset default 2D camera mode
// pub fn set_default_camera() {
//     let context = get_context();

//     // flush previous camera draw calls
//     context.perform_render_passes();

//     context.gl.render_pass(None);
//     context.gl.depth_test(false);
//     context.camera_matrix = None;
// }
