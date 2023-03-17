use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);

        set_camera(&Camera2D::from_display_rect(Rect::new(0., 0., 1., 1.)));
        draw_rectangle(0.5, 0.5, 0.5, 0.5, GREEN);

        set_camera(&Camera2D::from_display_rect(Rect::new(3., 3., 6., 6.)));
        draw_rectangle(3., 3., 3., 3., RED);

        next_frame().await
    }
}
