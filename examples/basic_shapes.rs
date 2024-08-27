use macroquad::window::next_frame;
use quad_gl::{color::*, math::*, shapes::*};

async fn game(ctx: macroquad::Context) {
    let mut canvas1 = ctx.new_canvas();
    let mut canvas2 = ctx.new_canvas();
    let mut canvas3 = ctx.new_canvas();

    // canvas1 is a static background canvas.
    // It will be never updated.
    canvas1.draw(Rectangle::new(100.0, 100.0), vec2(0.0, 0.0), RED);
    canvas1.draw(Text::new("HELLO WORLD", 30), vec2(300.0, 300.0), BLACK);

    loop {
        let t = ctx.time_since_start();

        // some fake animations
        let p1 = vec2(
            (t * 0.1).sin() * 400.0 + 400.0,
            (t * 0.1).cos() * 200.0 + 200.0,
        );
        let p2 = vec2((t * 3.0).sin() * 400.0 + 800.0, t.cos() * 200.0 + 400.0);

        ctx.clear_screen(WHITE);

        // canvas2 is an "additive" canvas. It holds its previous state
        // and will get an additiona red circle each frame.
        canvas2.draw(Circle::new(10.0), p1, RED);

        // canvas3 is a "dynamic" canvas. starts from scratch each frame.
        // Useful for animated content.
        canvas3.clear();
        canvas3.draw(Circle::new(10.0), p2, BLUE);

        // .draws order defines "Z" order, here canvas1 content will be
        // on the background and canvas3's blue circle will be on top of everything
        ctx.draw_canvas(&mut canvas1);
        ctx.draw_canvas(&mut canvas2);
        ctx.draw_canvas(&mut canvas3);

        next_frame().await;
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
