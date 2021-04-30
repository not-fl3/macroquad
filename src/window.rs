//! Window and associated to window rendering context related functions.

use crate::get_context;

use crate::color::Color;
use miniquad::PassAction;

// miniquad is re-exported for the use in combination with `get_internal_gl`
pub use miniquad;

/// Block execution until the next frame.
pub fn next_frame() -> crate::exec::FrameFuture {
    crate::exec::FrameFuture
}

pub fn clear_background(color: Color) {
    let context = get_context();

    // all drawcalls are batched
    // and batching is not clear-friendly
    // so as a workaround we do immediate render pass with clear color
    let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);
    if let Some(current_pass) = context.draw_context.current_pass {
        context.quad_context.begin_pass(current_pass, clear);
    } else {
        context.quad_context.begin_default_pass(clear);
    }
    context.quad_context.end_render_pass();

    context.draw_context.gl.clear_draw_calls();
}

pub struct InternalGlContext<'a> {
    pub quad_context: &'a mut miniquad::Context,
    pub quad_gl: &'a mut crate::quad_gl::QuadGl,
}

impl<'a> InternalGlContext<'a> {
    /// Draw all the batched stuff and reset the internal state cache
    /// May be helpful for combining macroquad's drawing with raw miniquad/opengl calls
    pub fn flush(&mut self) {
        let context = get_context();

        context
            .draw_context
            .perform_render_passes(&mut self.quad_context);
    }
}

pub unsafe fn get_internal_gl<'a>() -> InternalGlContext<'a> {
    let context = get_context();

    InternalGlContext {
        quad_context: &mut context.quad_context,
        quad_gl: &mut context.draw_context.gl,
    }
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height
}
