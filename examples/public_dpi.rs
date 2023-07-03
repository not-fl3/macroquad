use macroquad::{prelude::*, get_dpi_scale};

#[macroquad::main("Get DPI")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);

        draw_text(get_dpi_scale().to_string().as_str(), 10.0, 20.0, 30.0, BLACK);

        next_frame().await
    }
}
