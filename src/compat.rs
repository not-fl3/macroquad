//! miniquad-0.4 emulation

pub use crate::window::next_frame;
pub use quad_gl::color::*;

use std::cell::RefCell;

pub struct CompatContext {
    ctx: crate::Context,
    canvas: quad_gl::sprite_batcher::SpriteBatcher,
}

thread_local! {
    pub static CTX: RefCell<Option<CompatContext>> = { RefCell::new(None) };
}

fn with_ctx<F: Fn(&mut CompatContext)>(f: F) {
    CTX.with_borrow_mut(|v| f(v.as_mut().unwrap()));
}
pub fn init_compat_mode(ctx: crate::Context) {
    let canvas = ctx.quad_gl.new_canvas();

    CTX.with_borrow_mut(|v| {
        *v = Some(CompatContext { ctx, canvas });
    });
}

pub(crate) fn end_frame() {
    if CTX.with_borrow(|ctx| ctx.is_some()) {
        with_ctx(|ctx| {
            ctx.canvas.draw();
            ctx.canvas.reset();
        });
    }
}
pub fn clear_background(color: Color) {
    with_ctx(|ctx| {
        ctx.ctx
            .quad_ctx
            .lock()
            .unwrap()
            .clear(Some((1., 1., 1., 1.)), None, None)
    });
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    with_ctx(|ctx| ctx.canvas.draw_line(x1, y1, x2, y2, thickness, color));
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    with_ctx(|ctx| ctx.canvas.draw_rectangle(x, y, w, h, color));
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    with_ctx(|ctx| ctx.canvas.draw_circle(x, y, r, color));
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    with_ctx(|ctx| ctx.canvas.draw_text(text, x, y, font_size, color));
}

pub fn screen_width() -> f32 {
    crate::window::screen_width()
}

pub fn screen_height() -> f32 {
    crate::window::screen_height()
}
