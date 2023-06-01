use macroquad::prelude::*;

#[macroquad::main("NPatch")]
async fn main() {
    let texture = Texture2D::from_file_with_format(include_bytes!("npatch.png"), None);

    let npatch = NPatch {
        top: 6.0,
        bottom: 6.0,
        left: 6.0,
        right: 6.0,
    };

    let mut rect_dest = Rect::new(50.0, 50.0, 25.0, 30.0);

    loop {
        clear_background(LIGHTGRAY);

        let (mouse_x, mouse_y) = mouse_position();

        rect_dest.w = mouse_x - rect_dest.x;
        rect_dest.h = mouse_y - rect_dest.y;

        if rect_dest.w < texture.width() {
            rect_dest.w = texture.width();
        }
        if rect_dest.h < texture.height() {
            rect_dest.h = texture.height();
        }

        draw_npatch(npatch, texture, rect_dest, WHITE);
        draw_text(
            "resize the n-patch with the mouse",
            20.0,
            20.0,
            20.0,
            DARKGRAY,
        );
        next_frame().await
    }
}
