use miniquad::*;
use quad_gl::*;

struct Stage {
    gl: QuadGl,
}
impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear(Some((0., 1., 0., 1.)), None, None);

        self.gl.draw_mode(DrawMode::Lines);
        self.gl.geometry(
            &[
                Vertex::new(0., -0.5, 0., 0., 0., BLUE),
                Vertex::new(0.5, 0.5, 0., 0., 0., RED),
                Vertex::new(-0.5, 0.5, 0., 0., 0., GREEN),
            ],
            &[0, 1, 2, 0, 1, 2],
        );
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
