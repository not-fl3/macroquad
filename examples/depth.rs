use macroquad::prelude::*;

#[macroquad::main("Sprites with Depth")]
async fn main() {
    let ferris = load_texture("examples/ferris.png").await.unwrap();

    let mut depth = 0f32;

    loop {
        let scroll = mouse_wheel().1;
        if scroll < 0.0 || is_key_pressed(KeyCode::Down) {
            depth -= 0.1;
        } else if scroll > 0.0 || is_key_pressed(KeyCode::Up) {
            depth += 0.1;
        }
        depth = depth.clamp(-1.0, 1.0);

        clear_background(LIGHTGRAY);

        // Draw some rust icons from left to right
        for z in -5..=5 {
            let x = screen_width() / 2.0 - (screen_width() / 10.0) * -z as f32;
            let y = screen_height() / 2.0;

            let size = screen_width() / 12.0;

            let brightness = (z + 5 + 1) as f32 / 11.0;
            let color = Color::new(brightness, brightness, brightness, 0.7);

            draw_texture_ex(
                ferris,
                x,
                y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    depth: z as f32 / 10.0,
                    ..Default::default()
                },
            );
        }

        let (mx, my) = mouse_position();
        let size = screen_width() / 6.0;
        draw_texture_ex(
            ferris,
            mx,
            my,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                depth,
                ..Default::default()
            },
        );

        draw_text(
            &format!("Ferris depth: {:.1}. Scroll or arrow keys to change", depth),
            10.0,
            40.0,
            32.0,
            BLACK,
        );

        next_frame().await
    }
}
