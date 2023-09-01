use macroquad::prelude::*;

async fn game(ctx: macroquad::Context3) {
    let texture: Texture2D = ctx
        .load_texture("/home/fl3/tmp/progress.jpg")
        .await
        .unwrap();

    let texture2: Texture2D = ctx
        .load_texture2("/home/fl3/tmp/progress.jpg")//Default_metalRoughness.jpg")
        .await
        .unwrap();

    let mut canvas = ctx.new_sprite_layer();
    loop {
        canvas.draw_texture(texture.clone(), 0., 0., WHITE);
        canvas.draw_texture(texture2.clone(), 500., 0., WHITE);
        canvas.draw();
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
