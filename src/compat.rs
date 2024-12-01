//! miniquad-0.4 emulation

pub use quad_gl::color::*;

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
    task::Waker,
};

use crate::exec::FrameFuture;

pub struct CompatContext {
    quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    frame_wakers: Arc<Mutex<Vec<Waker>>>,
    canvas: quad_gl::sprite_batcher::SpriteBatcher,
}

thread_local! {
    pub static CTX: RefCell<Option<CompatContext>> = { RefCell::new(None) };
}

pub fn next_frame() -> FrameFuture {
    with_ctx(|ctx| FrameFuture {
        frame_wakers: Some(ctx.frame_wakers.clone()),
    })
}

fn with_ctx<T, F: Fn(&mut CompatContext) -> T>(f: F) -> T {
    CTX.with_borrow_mut(|v| f(v.as_mut().unwrap()))
}
pub fn init_compat_mode(ctx: &crate::Context) {
    let canvas = ctx.new_canvas();
    let quad_ctx = ctx.quad_ctx.clone();
    let frame_wakers = ctx.frame_wakers.clone();

    CTX.with_borrow_mut(|v| {
        *v = Some(CompatContext {
            quad_ctx,
            canvas,
            frame_wakers,
        });
    });
}

pub(crate) fn end_frame() {
    if CTX.with_borrow(|ctx| ctx.is_some()) {
        with_ctx(|ctx| {
            ctx.canvas.blit(None);
            ctx.canvas.reset();
        });
    }
}
pub fn clear_background(color: Color) {
    with_ctx(|ctx| {
        ctx.quad_ctx
            .lock()
            .unwrap()
            .clear(Some((1., 1., 1., 1.)), None, None)
    });
}

// pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
//     with_ctx(|ctx| ctx.canvas.draw_line(x1, y1, x2, y2, thickness, color));
// }

// pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
//     with_ctx(|ctx| ctx.canvas.draw_rectangle(x, y, w, h, color));
// }

// pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
//     with_ctx(|ctx| ctx.canvas.draw_circle(x, y, r, color));
// }

// pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
//     with_ctx(|ctx| ctx.canvas.draw_text(text, x, y, font_size, color));
// }

pub fn screen_width() -> f32 {
    crate::window::screen_width()
}

pub fn screen_height() -> f32 {
    crate::window::screen_height()
}
