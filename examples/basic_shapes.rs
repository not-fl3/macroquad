use macroquad::*;

fn main() {
    Window::init("BasicShapes").main_loop(|| {
        clear_background(RED);

        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);
    });
}
