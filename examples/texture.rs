use macroquad::window::next_frame;
use quad_gl::{color::*, math::*, shapes::ShapeBuilder};

async fn game(ctx: macroquad::Context) {
    let texture = ctx
        .resources
        .load_texture("examples/ferris.png")
        .await
        .unwrap();

    let mut canvas = ctx.new_canvas();
    loop {
        ctx.clear_screen(WHITE);
        canvas.clear();
        ShapeBuilder::rectangle(vec2(texture.width() as f32, texture.height() as f32))
            .texture(&texture)
            .position(vec2(ctx.screen_width() / 2.0, ctx.screen_height() / 2.0))
            .draw(&mut canvas);
        canvas.draw();
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
