use macroquad::prelude::*;

fn short_angle_dist(a0: f32, a1: f32) -> f32 {
    let max = 360.0;
    let da = (a1 - a0) % max;
    2.0 * da % max - da
}

fn angle_lerp(a0: f32, a1: f32, t: f32) -> f32 {
    a0 + short_angle_dist(a0, a1) * t
}

fn draw_cross(x: f32, y: f32, color: Color) {
    let size = 0.1;
    let thickness = 0.005;
    draw_line(x - size, y, x + size, y, thickness, color);
    draw_line(x, y - size, x, y + size, thickness, color);
}

#[macroquad::main("Camera")]
async fn main() {
    let mut target = (0., 0.);
    let mut zoom = 1.0;
    let mut rotation = 0.0;
    let mut smooth_rotation: f32 = 0.0;
    let mut offset = (0., 0.);

    loop {
        if is_key_down(KeyCode::W) {
            target.1 -= 0.1;
        }
        if is_key_down(KeyCode::S) {
            target.1 += 0.1;
        }
        if is_key_down(KeyCode::A) {
            target.0 += 0.1;
        }
        if is_key_down(KeyCode::D) {
            target.0 -= 0.1;
        }
        if is_key_down(KeyCode::Left) {
            offset.0 -= 0.1;
        }
        if is_key_down(KeyCode::Right) {
            offset.0 += 0.1;
        }
        if is_key_down(KeyCode::Up) {
            offset.1 += 0.1;
        }
        if is_key_down(KeyCode::Down) {
            offset.1 -= 0.1;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Q) | is_key_down(KeyCode::Escape) {
            break;
        }

        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                // Normalize mouse wheel values is browser (chromium: 53, firefox: 3)
                #[cfg(target_arch = "wasm32")]
                let y = if y < 0.0 {
                    -1.0
                } else if y > 0.0 {
                    1.0
                } else {
                    0.0
                };
                if is_key_down(KeyCode::LeftControl) {
                    zoom *= 1.1f32.powf(y);
                } else {
                    rotation += 10.0 * y;
                    rotation = match rotation {
                        angle if angle >= 360.0 => angle - 360.0,
                        angle if angle < 0.0 => angle + 360.0,
                        angle => angle,
                    }
                }
            }
            _ => (),
        }

        smooth_rotation = angle_lerp(smooth_rotation, rotation, 0.1);

        clear_background(LIGHTGRAY);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            ..Default::default()
        });
        draw_cross(0., 0., RED);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            ..Default::default()
        });
        draw_cross(0., 0., GREEN);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            zoom: vec2(zoom, zoom * screen_width() / screen_height()),
            ..Default::default()
        });
        draw_cross(0., 0., BLUE);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            zoom: vec2(zoom, zoom * screen_width() / screen_height()),
            offset: vec2(offset.0, offset.1),
            ..Default::default()
        });

        // Render some primitives in camera space
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

        // Back to screen space, render some text
        set_default_camera();
        draw_text(
            format!("target (WASD keys) = ({:+.2}, {:+.2})", target.0, target.1).as_str(),
            10.0,
            10.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("rotation (mouse wheel) = {} degrees", rotation).as_str(),
            10.0,
            25.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("zoom (ctrl + mouse wheel) = {:.2}", zoom).as_str(),
            10.0,
            40.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("offset (arrow keys) = ({:+.2}, {:+.2})", offset.0, offset.1).as_str(),
            10.0,
            55.0,
            15.0,
            BLACK,
        );
        draw_text("HELLO", 30.0, 200.0, 30.0, BLACK);

        next_frame().await
    }
}
