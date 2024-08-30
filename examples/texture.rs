use macroquad::window::next_frame;
use quad_gl::{color::*, math::*, shapes::*};

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
        canvas.draw(
            Sprite::new(&texture),
            vec2(
                ctx.screen_width() - texture.width() as f32,
                ctx.screen_height() - texture.height() as f32,
            ) / 2.,
            WHITE,
        );
        ctx.blit_canvas(&mut canvas);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
