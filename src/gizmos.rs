pub use crate::{
    math::{vec2, vec3, Vec3},
    window::next_frame,
};
pub use quad_gl::{
    color::*,
    draw_calls_batcher::{DrawCallsBatcher, DrawMode, Vertex},
};

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

struct Line {
    persist: bool,
    p0: Vec3,
    p1: Vec3,
}

pub struct Gizmos {
    quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    canvas: quad_gl::sprite_batcher::SpriteBatcher,
    lines: Vec<Line>,
}

thread_local! {
    pub static CTX: RefCell<Option<Gizmos>> = { RefCell::new(None) };
}

fn with_ctx<F: Fn(&mut Gizmos)>(f: F) {
    CTX.with_borrow_mut(|v| f(v.as_mut().unwrap()));
}
pub fn init_gizmos(ctx: &crate::Context) {
    let canvas = ctx.new_canvas();
    let quad_ctx = ctx.quad_ctx.clone();

    CTX.with_borrow_mut(|v| {
        *v = Some(Gizmos {
            quad_ctx,
            canvas,
            lines: vec![],
        });
    });
}

fn draw_line(gl: &mut DrawCallsBatcher, p0: Vec3, p1: Vec3) {
    let uv = vec2(0., 0.);
    let indices = [0, 1];

    let line = [
        Vertex::new2(vec3(p0.x, p0.y, p0.z), uv, BLUE),
        Vertex::new2(vec3(p1.x, p1.y, p1.z), uv, BLUE),
    ];
    gl.texture(None);
    gl.draw_mode(DrawMode::Lines);
    gl.geometry(&line[..], &indices);
}

pub fn draw_gizmos(camera: &quad_gl::camera::Camera) {
    if CTX.with_borrow(|ctx| ctx.is_some()) {
        with_ctx(|ctx| {
            let mut gl = ctx.canvas.gl();
            gl.depth_test(true);
            for line in &mut ctx.lines {
                draw_line(gl, line.p0, line.p1);
            }

            let (proj, view) = camera.proj_view();
            ctx.canvas.set_override_matrix(proj * view);
            ctx.canvas.blit(None);
            ctx.canvas.reset();

            ctx.lines.retain(|line| line.persist);
        });
    }
}

pub fn gizmos_add_line(persist: bool, p0: Vec3, p1: Vec3) {
    with_ctx(|ctx| {
        ctx.lines.push(Line { persist, p0, p1 });
    });
}
