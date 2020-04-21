use macroquad::*;

#[macroquad::main("Camera")]
async fn main() {
    loop {
        clear_background(RED);

        // Render some primitives in camera space

        begin_mode_2d(Camera2D {
            zoom: vec2(1., screen_width() / screen_height()),
            ..Default::default()
        });
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);
        end_mode_2d();

        // Back to screen space, render some text

        draw_text("HELLO", 30.0, 200.0, 30.0, BLACK);

        next_frame().await
    }
}
