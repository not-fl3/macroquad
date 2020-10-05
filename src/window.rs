//! Window and associated to window rendering context related functions.

use crate::get_context;

use quad_gl::Color;
use miniquad::PassAction;

/// Block execution until the next frame.
pub fn next_frame() -> crate::exec::FrameFuture {
    crate::exec::FrameFuture
}

pub fn clear_background(color: Color) {
    let context = get_context();

    // all drawcalls are batched
    // and batching is not clear-friendly
    // so as a workaround we do immediate render pass with clear color
    let clear = PassAction::clear_color(
        color.0[0] as f32 / 255.,
        color.0[1] as f32 / 255.,
        color.0[2] as f32 / 255.,
        color.0[3] as f32 / 255.,
    );
    if let Some(current_pass) = context.draw_context.current_pass {
        context.quad_context.begin_pass(current_pass, clear);
    } else {
        context.quad_context.begin_default_pass(clear);
    }
    context.quad_context.end_render_pass();

    context.draw_context.gl.clear_draw_calls();
}

pub unsafe fn get_internal_gl<'a>() -> &'a mut quad_gl::QuadGl {
    let context = &mut get_context().draw_context;

    &mut context.gl
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height
}

