use macroquad::prelude::*;

#[macroquad::main("High DPI Test")]
async fn main() {
    loop {
        clear_background(BLACK);

        set_default_camera();

        //Draw some rectangles to demonstrate that they look the same on both high DPI and low DPI screens
        draw_rectangle(0f32, 0f32, screen_width(), screen_height(), RED);
        draw_rectangle(0f32, 0f32, screen_width() / 2., screen_height() / 2., GREEN);
        
        next_frame().await;
    }
}
