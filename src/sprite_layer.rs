use crate::{quad_gl::QuadGl, text};

use std::sync::{Arc, Mutex};

pub struct SpriteLayer {
    pub(crate) quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    pub(crate) fonts_storage: Arc<Mutex<text::FontsStorage>>,
    pub(crate) quad_gl: QuadGl,
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
        }
    }

    pub fn gl(&mut self) -> &mut QuadGl {
        &mut self.quad_gl
    }

    pub fn draw(&mut self) {
        let mut ctx = self.quad_ctx.lock().unwrap();

        let (width, height) = miniquad::window::screen_size();

        let screen_mat = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
        self.quad_gl.draw(&mut **ctx, screen_mat);
    }
}
