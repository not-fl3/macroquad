use crate::{quad_gl::QuadGl, text};

use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub struct SpriteLayer {
    pub(crate) quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    pub(crate) fonts_storage: Arc<Mutex<text::FontsStorage>>,
    pub(crate) quad_gl: QuadGl,
    pub(crate) axis: Axis,
}

impl SpriteLayer {
    pub(crate) fn new(
        quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
        fonts_storage: Arc<Mutex<text::FontsStorage>>,
    ) -> SpriteLayer {
        let mut ctx = quad_ctx.lock().unwrap();

        let quad_gl = QuadGl::new(&mut **ctx);
        SpriteLayer {
            quad_ctx: quad_ctx.clone(),
            fonts_storage: fonts_storage.clone(),
            quad_gl,
            axis: Axis::Z,
        }
    }

    pub fn set_axis(&mut self, axis: Axis) {
        self.axis = axis;
    }

    pub fn gl(&mut self) -> &mut QuadGl {
        &mut self.quad_gl
    }

    pub fn reset(&mut self) {
        self.quad_gl.reset()
    }

    pub fn clear(&mut self, color: crate::Color) {
        self.quad_gl
            .clear(self.quad_ctx.lock().unwrap().as_mut(), color)
    }

    pub fn wtf(&mut self, mat: crate::math::Mat4) {
        self.quad_gl.push_model_matrix(mat);
    }
    pub fn draw(&mut self) {
        let mut ctx = self.quad_ctx.lock().unwrap();

        let (width, height) = miniquad::window::screen_size();

        let screen_mat = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
        self.quad_gl.draw(&mut **ctx, screen_mat, None);
    }

    pub fn draw2(&mut self, camera: &crate::camera::Camera) {
        let mut ctx = self.quad_ctx.lock().unwrap();

        let (proj, view) = camera.proj_view();
        self.quad_gl.draw(
            &mut **ctx,
            proj * view,
            camera.render_target.clone().map(|t| t.render_pass),
        );
    }
}
