use std::f32::consts::PI;

use macroquad::prelude::*;

const VIRTUAL_WIDTH: f32 = 640.0;
const VIRTUAL_HEIGHT: f32 = 360.0;

#[macroquad::main("Balls")]
async fn main() {
    // Setup 'render_target', used to hold the rendering result so we can resize it
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    // Setup camera for the virtual screen, that will render to 'render_target'
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target.clone());

    let mut balls = Vec::with_capacity(256);

    for _ in 0..8 {
        balls.push(Ball::default());
    }

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Get required scaling value
        let scale: f32 = f32::min(
            screen_width() / VIRTUAL_WIDTH,
            screen_height() / VIRTUAL_HEIGHT,
        );

        add_balls(scale, &mut balls);
        let result = remove_balls(scale, &mut balls);

        // ------------------------------------------------------------------------
        // Begin drawing the virtual screen to 'render_target'
        // ------------------------------------------------------------------------
        set_camera(&render_target_cam);

        clear_background(DARKGRAY);

        process_balls(&mut balls);

        draw_text(
            &format!("{}/{} balls", balls.len(), balls.capacity()),
            0.0,
            10.0,
            16.0,
            WHITE,
        );

        if let Some((radius, virtual_mouse_pos)) = result {
            draw_circle(virtual_mouse_pos.x, virtual_mouse_pos.y, radius, WHITE);
        }

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

fn add_balls(scale: f32, balls: &mut Vec<Ball>) {
    // Mouse position in the virtual screen
    let virtual_mouse_pos = Vec2 {
        x: (mouse_position().0 - (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5) / scale,
        y: (mouse_position().1 - (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5) / scale,
    };

    if is_mouse_button_down(MouseButton::Left) {
        let count = if is_key_down(KeyCode::LeftShift) {
            10
        } else {
            1
        };

        for _ in 0..count {
            balls.push(Ball {
                position: virtual_mouse_pos,
                ..Default::default()
            });
        }
    }
}

fn remove_balls(scale: f32, balls: &mut Vec<Ball>) -> Option<(f32, Vec2)> {
    // Mouse position in the virtual screen
    let virtual_mouse_pos = Vec2 {
        x: (mouse_position().0 - (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5) / scale,
        y: (mouse_position().1 - (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5) / scale,
    };

    if is_mouse_button_down(MouseButton::Right) {
        let radius = 16.0;

        'outer: loop {
            for (index, ball) in balls.iter().enumerate() {
                if ball.position.distance(virtual_mouse_pos) < radius {
                    balls.remove(index);
                    continue 'outer;
                }
            }

            break 'outer;
        }

        return Some((radius, virtual_mouse_pos));
    }

    None
}

fn process_balls(balls: &mut Vec<Ball>) {
    for ball in balls {
        // Physics
        let offset = ball.velocity * get_frame_time();
        ball.position += offset;

        if bounce(ball) {
            ball.velocity *= 0.9;
            // Fix: prevent balls from getting stuck by undoing movement
            ball.position -= offset;
        }

        // Graphics
        draw_circle(ball.position.x, ball.position.y, 8.0, ball.color);
    }
}

fn bounce(ball: &mut Ball) -> bool {
    let mut has_bounced = false;

    if ball.position.x < 0.0 || ball.position.x > VIRTUAL_WIDTH {
        ball.velocity.x = -ball.velocity.x;
        has_bounced = true;
    }

    if ball.position.y < 0.0 || ball.position.y > VIRTUAL_HEIGHT {
        ball.velocity.y = -ball.velocity.y;
        has_bounced = true;
    }

    has_bounced
}

#[derive(Clone, Copy)]
struct Ball {
    position: Vec2,
    velocity: Vec2,
    color: Color,
}

impl Default for Ball {
    fn default() -> Self {
        let angle = (rand::rand() % 360) as f32 / 180.0 * PI;

        Self {
            position: Vec2::new(VIRTUAL_HEIGHT, VIRTUAL_HEIGHT) / 2.0,
            velocity: Vec2::from_angle(angle) * 128.0,
            color: RED,
        }
    }
}
