use macroquad::prelude::*;

#[macroquad::main("Texture")]
async fn main() {
    let mut texture: Texture2D = load_texture("examples/ferris.png").await.unwrap();
    texture.set_wrap(TextureWrap::Repeat);

    loop {
        clear_background(WHITE);

        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(700.0, 700.0)),
                ..Default::default()
            },
        );

        next_frame().await
    }
}
