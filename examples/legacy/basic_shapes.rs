use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);

        let mut c = scene_graph().fullscreen_canvas();

        c.draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        c.draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        c.draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        c.draw_text("HELLO", 20.0, 20.0, 30.0, DARKGRAY);

        scene_graph().draw_canvas(c);

        next_frame().await
    }
}
