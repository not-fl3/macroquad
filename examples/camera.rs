use macroquad::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Camera".to_string(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(RED);

        // Render some primitives in camera space

        set_camera(Camera2D {
            zoom: vec2(1., screen_width() / screen_height()),
            ..Default::default()
        });
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

        // Back to screen space, render some text

        set_default_camera();
        draw_text("HELLO", 30.0, 200.0, 30.0, BLACK);

        next_frame().await
    }
}
