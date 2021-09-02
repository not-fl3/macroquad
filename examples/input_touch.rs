use macroquad::prelude::*;

#[macroquad::main("InputTouch")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);

        for touch in touches() {
            let (fill_color, size) = match touch.phase {
                TouchPhase::Started => (GREEN, 80.0),
                TouchPhase::Stationary => (WHITE, 60.0),
                TouchPhase::Moved => (YELLOW, 60.0),
                TouchPhase::Ended => (BLUE, 80.0),
                TouchPhase::Cancelled => (BLACK, 80.0),
            };
            draw_circle(touch.position.x, touch.position.y, size, fill_color);
        }

        draw_text("touch the screen!", 20.0, 20.0, 20.0, DARKGRAY);
        next_frame().await
    }
}
