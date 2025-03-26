use macroquad::prelude::*;

#[macroquad::main("Upscaling")]
async fn main() {
    let texture: Texture2D = load_texture("examples/rustacean_happy.png").await.unwrap();
    let double = Texture2D::from_image(&texture.get_texture_data().upscale(2));
    let triple = Texture2D::from_image(&texture.get_texture_data().upscale(3));

    loop {
        clear_background(LIGHTGRAY);
        draw_texture(&texture, 40., 40., WHITE);
        draw_texture(&double, 140., 140., WHITE);
        draw_texture(&triple, 240., 240., WHITE);

        next_frame().await
    }
}
