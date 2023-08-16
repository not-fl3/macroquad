use macroquad::prelude::*;

const VIRTUAL_WIDTH: f32 = 1280.0;
const VIRTUAL_HEIGHT: f32 = 720.0;

#[macroquad::main("Letterbox")]
async fn main() {
    // Setup 'render_target', used to hold the rendering result so we can resize it
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Linear);

    // Setup camera for the virtual screen, that will render to 'render_target'
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target.clone());

    loop {
        // Get required scaling value
        let scale: f32 = f32::min(
            screen_width() / VIRTUAL_WIDTH,
            screen_height() / VIRTUAL_HEIGHT,
        );

        // Mouse position in the virtual screen
        let virtual_mouse_pos = Vec2 {
            x: (mouse_position().0 - (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5) / scale,
            y: (mouse_position().1 - (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5) / scale,
        };

        // ------------------------------------------------------------------------
        // Begin drawing the virtual screen to 'render_target'
        // ------------------------------------------------------------------------
        set_camera(&render_target_cam);

        clear_background(LIGHTGRAY);

        draw_text("Hello Letterbox", 20.0, 20.0, 30.0, DARKGRAY);
        draw_circle(VIRTUAL_WIDTH / 2.0 - 65.0, VIRTUAL_HEIGHT / 2.0, 35.0, RED);
        draw_circle(VIRTUAL_WIDTH / 2.0 + 65.0, VIRTUAL_HEIGHT / 2.0, 35.0, BLUE);
        draw_circle(
            VIRTUAL_WIDTH / 2.0,
            VIRTUAL_HEIGHT / 2.0 - 65.0,
            35.0,
            YELLOW,
        );

        draw_circle(virtual_mouse_pos.x, virtual_mouse_pos.y, 15.0, BLACK);

        // ------------------------------------------------------------------------
        // Begin drawing the window screen
        // ------------------------------------------------------------------------
        set_default_camera();

        clear_background(BLACK); // Will be the letterbox color

        // Draw 'render_target' to window screen, porperly scaled and letterboxed
        draw_texture_ex(
            &render_target.texture,
            (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5,
            (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                ..Default::default()
            },
        );

        next_frame().await;
    }
}
