use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);

        draw_pretty_circle(
            screen_width() * 0.5 - 250.,
            screen_height() * 0.5 - 250.,
            500.,
            500.,
            BLUE,
        );

        draw_text("PRETTY CIRCLE", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
