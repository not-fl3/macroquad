//! this is legacy and going to disappear soon

use crate::quad_gl::QuadGl;

pub use crate::quad_gl::{colors::*, Color, DrawMode, FilterMode};
pub use crate::texture::Texture2D;

use glam::Mat4;

pub struct DrawContext {
    pub(crate) gl: QuadGl,
    pub(crate) camera_matrix: Option<Mat4>,
    pub(crate) current_pass: Option<miniquad::RenderPass>,
}

impl DrawContext {
    pub fn new(ctx: &mut miniquad::Context) -> DrawContext {
        DrawContext {
            camera_matrix: None,
            gl: QuadGl::new(ctx),
            current_pass: None,
        }
    }

    pub(crate) fn projection_matrix(&self, ctx: &mut miniquad::Context) -> glam::Mat4 {
        let (width, height) = ctx.screen_size();

        if let Some(matrix) = self.camera_matrix {
            matrix
        } else {
            glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.)
        }
    }

    pub(crate) fn perform_render_passes(&mut self, ctx: &mut miniquad::Context) {
        let matrix = self.projection_matrix(ctx);

        self.gl.draw(ctx, matrix);
    }
}
