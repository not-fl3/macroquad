//! this is legacy and going to disappear soon

use quad_gl::QuadGl;

pub use quad_gl::{colors::*, Color, DrawMode, FilterMode, Image, Texture2D};

use glam::Mat4;

pub struct DrawContext {
    pub(crate) gl: QuadGl,
    pub(crate) camera_matrix: Option<Mat4>,
    pub(crate) current_pass: Option<miniquad::RenderPass>,
}

impl DrawContext {
    pub fn new(ctx: &mut miniquad::Context) -> DrawContext {
        let mut draw_context = DrawContext {
            camera_matrix: None,
            gl: QuadGl::new(ctx),
            current_pass: None,
        };

        draw_context.update_projection_matrix(ctx);

        draw_context
    }

    pub(crate) fn perform_render_passes(&mut self, ctx: &mut miniquad::Context) {
        self.gl.draw(ctx);
    }

    pub(crate) fn update_projection_matrix(&mut self, ctx: &mut miniquad::Context) {
        let (width, height) = ctx.screen_size();

        let projection = if let Some(matrix) = self.camera_matrix {
            matrix
        } else {
            glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.)
        };

        self.gl.set_projection_matrix(projection);
    }
}
