use macroquad::*;

fn main() {
    let mut x = 0.;
    let mut y = 0.;

    Window::init("Input")
        .on_init(|| {
            x = screen_width() / 2.0;
            y = screen_height() / 2.0;
        })
        .main_loop(|| {
            clear_background(RED);

            if is_key_down(KeyCode::Right) {
                x += 1.0;
            }
            if is_key_down(KeyCode::Left) {
                x -= 1.0;
            }
            if is_key_down(KeyCode::Down) {
                y += 1.0;
            }
            if is_key_down(KeyCode::Up) {
                y -= 1.0;
            }

            draw_circle(x, y, 15.0, YELLOW);
            draw_text("move the ball with arrow keys", 20.0, 20.0, 20.0, DARKGRAY);
        });
}
