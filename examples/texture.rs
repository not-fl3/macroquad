use macroquad::window::next_frame;
use quad_gl::{color::*, math::*, shapes::ShapeBuilder, texture::Texture2D};

async fn game(ctx: macroquad::Context) {
    let texture: Texture2D = ctx
        .resources
        .load_texture("examples/ferris.png")
        .await
        .unwrap();

    let mut canvas = ctx.new_sprite_layer();
    loop {
        canvas.clear(WHITE);
        ShapeBuilder::rectangle(vec2(texture.width(), texture.height()))
            .texture(&texture)
            .position(vec2(ctx.screen_width() / 2.0, ctx.screen_height() / 2.0))
            .draw(&mut canvas);
        ctx.draw_canvas(&mut canvas);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
