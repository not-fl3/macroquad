use miniquad::*;
use quad_gl::*;

struct Stage {
    gl: QuadGl,
}

fn rect(gl: &mut QuadGl, w: f32, h: f32) {
    gl.geometry(
        &[
            Vertex::new(-w / 2., h, 0., 0., 0., RED),
            Vertex::new(w / 2., h, 0., 0., 0., RED),
            Vertex::new(-w / 2., 0., 0., 0., 0., RED),
            Vertex::new(w / 2., 0., 0., 0., 0., RED),
        ],
        &[0, 1, 2, 1, 2, 3],
    );
}

fn tree(gl: &mut QuadGl, time: f64, deep: u32, angle: f32, tall: f32) {
    if deep >= 8 {
        return;
    }

    // root
    rect(gl, 0.01, tall);

    gl.push_model_matrix(glam::Mat4::from_translation(glam::vec3(0., tall, 0.)));

    // right leaf
    gl.push_model_matrix(glam::Mat4::from_rotation_z(angle + time.sin() as f32 * 0.1));
    tree(gl, time, deep + 1, angle * 0.7, tall * 0.8);
    gl.pop_model_matrix();

    // left leaf
    gl.push_model_matrix(glam::Mat4::from_rotation_z(
        -angle - time.cos() as f32 * 0.1,
    ));
    tree(gl, time, deep + 1, angle * 0.7, tall * 0.8);
    gl.pop_model_matrix();

    gl.pop_model_matrix();
}
impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((0., 1., 0., 1.)), None, None);

        self.gl
            .push_model_matrix(glam::Mat4::from_translation(glam::vec3(0., -0.5, 0.)));
        tree(&mut self.gl, miniquad::date::now(), 0, 1., 0.3);
        self.gl.pop_model_matrix();

        self.gl.draw(ctx);
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(
            Stage {
                gl: QuadGl::new(&mut ctx),
            },
            ctx,
        )
    });
}
