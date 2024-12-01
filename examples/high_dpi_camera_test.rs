use macroquad::prelude::*;

#[macroquad::main("High DPI Camera Test")]
async fn main() {
    loop {
        clear_background(BLACK);

        //Set the camera to 100px in on each side to show the difference in x, y, width and height
        set_camera(&Camera2D {
            viewport: Some((100, 100, screen_width() as i32 - 200, screen_height() as i32 - 200)),
            offset: vec2(-1., -1.),
            ..Default::default()
        });

        //Draw some rectangles to demonstrate that they look the same on both high DPI and low DPI screens
        //Width and height of 2 makes up the entire viewable camera area
        draw_rectangle(0f32, 0f32, 2., 2., RED);
        draw_rectangle(0f32, 0f32, 1., 1., GREEN);
        
        next_frame().await;
    }
}
