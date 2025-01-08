use macroquad::prelude::*;
extern crate micromath;
use micromath::F32Ext;

fn clip(pos: f32, bound: f32, velocity: f32) -> f32 {
    if pos > bound {
        let delta = (pos - bound).abs();
        let mut new_velocity = velocity;
        if velocity > 0.0 {
            new_velocity = -velocity
        }
        if (new_velocity.abs() + BALL_RADIUS) < delta {
            return -delta / get_fps() as f32 * 60.0;
        }
        return new_velocity * FLOOR_LOSS;
    }
    velocity
}

fn clip_minus(pos: f32, bound: f32, velocity: f32) -> f32 {
    return -clip(bound, pos, -velocity);
}

#[derive(Debug, Clone, Copy)]
pub struct Ball {
    id: usize,
    x: f32,
    y: f32,
    x_vel: f32,
    y_vel: f32,
    color: Color,
}

// Customizable
const FLOOR_LOSS: f32 = 0.9;
const DRAG: f32 = 0.998;
const GRAVITY: f32 = 1.0;
const MAX_VEL: f32 = 40.0;

const BALL_RADIUS: f32 = 5.0;

// Please, don't change
static mut BALL_ID: usize = 0;
const BALL_RADIUS_SQR: f32 = BALL_RADIUS * BALL_RADIUS;

impl Ball {
    fn new(x: f32, y: f32, seed: usize) -> Ball {
        let id;
        unsafe {
            id = BALL_ID;
            BALL_ID += 1;
        }

        let seed = seed % (usize::MAX / 64);
        let random = (32_1239 * seed * seed * seed * 17 + id) % 1000;
        let x_vel = random % 9;
        let x_vel = (x_vel as f32 - 4.0) / 1.5;

        return Ball {
            id,
            x,
            y,
            x_vel,
            y_vel: 0.0,
            color: Color::from_rgba(
                (random as usize * 47 % 255) as u8,
                (random as usize * 29 % 255) as u8,
                (random as usize * 101 % 255) as u8,
                255,
            ),
        };
    }

    fn tick(&mut self) {
        self.x_vel *= DRAG;
        self.y_vel *= DRAG;

        self.x += self.x_vel;
        self.y += self.y_vel;

        self.x = self.x.clamp(-3000.0, 3000.0);
        self.x = if self.x.is_normal() { self.x } else { 0.0 };
        self.y = self.y.clamp(-3000.0, 3000.0);
        self.y = if self.y.is_normal() { self.y } else { 0.0 };
        self.y_vel = self.y_vel.clamp(-MAX_VEL, MAX_VEL);
        self.y_vel = if self.y_vel.is_normal() {
            self.y_vel
        } else {
            0.0
        };
        self.x_vel = self.x_vel.clamp(-MAX_VEL, MAX_VEL);
        self.x_vel = if self.x_vel.is_normal() {
            self.x_vel
        } else {
            0.0
        };

        self.y_vel = clip(self.y, screen_height(), self.y_vel + GRAVITY);
        self.x_vel = clip(self.x, screen_width(), self.x_vel);
        self.x_vel = clip_minus(self.x, 0.0, self.x_vel);
        self.y_vel = clip_minus(self.y, -screen_height(), self.y_vel);
    }

    fn draw(&self) {
        draw_circle(self.x, self.y - BALL_RADIUS, BALL_RADIUS, self.color);
    }

    fn collide(&mut self, balls: &[Ball]) {
        unsafe {
            let skip = (BALL_ID / 1000).max(8);

            for other in balls {
                if other.id == self.id || other.id % 10 < skip {
                    continue;
                }

                let dx = self.x - other.x;
                let dy = self.y - other.y;
                let distance_sqr = dx * dx + dy * dy;

                if distance_sqr > BALL_RADIUS_SQR {
                    continue;
                }

                let distance = distance_sqr.sqrt();
                if distance == 0.0 {
                    continue;
                }
                let nx = dx / distance;
                let ny = dy / distance;

                let relative_velocity_x = self.x_vel - other.x_vel;
                let relative_velocity_y = self.y_vel - other.y_vel;
                let relative_velocity_normal = relative_velocity_x * nx + relative_velocity_y * ny;

                if relative_velocity_normal > 0.0 {
                    continue;
                }

                let impulse = (2.0 * relative_velocity_normal) / 2.0 * FLOOR_LOSS;

                self.x_vel -= (impulse * nx).clamp(-MAX_VEL, MAX_VEL);
                self.y_vel -= (impulse * ny).clamp(-MAX_VEL, MAX_VEL);
            }
        }
    }
}

#[macroquad::main("AM - Confetti")]
async fn main() {
    let mut balls: Vec<Ball> = Vec::new();
    let mut frame_count: usize = 0;
    loop {
        // Loop start
        frame_count += 1;
        clear_background(Color {
            r: 0.95,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        });

        // Handle Inputs
        if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            balls.push(Ball::new(mouse_x, mouse_y, frame_count));
            balls.push(Ball::new(mouse_x, mouse_y, frame_count));
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            let (mouse_x, mouse_y) = mouse_position();
            balls.push(Ball::new(mouse_x, mouse_y, frame_count))
        }
        if is_key_pressed(KeyCode::Space) {
            unsafe {
                BALL_ID = 0;
                balls.clear();
            }
        }

        // Handle Tick
        let balls_prev = balls.to_vec();
        for ball in &mut balls {
            ball.collide(&balls_prev);
            ball.tick();
            ball.draw();
        }

        unsafe {
            if BALL_ID == 0 {
                draw_text(
                    "Click anywhere to begin!",
                    screen_width() / 2.0 - 240.0,
                    screen_height() / 2.0,
                    48.0,
                    RED,
                );
            } else {
                draw_text(
                    &format!(
                        "Balls: {}\nFPS: {} {}",
                        BALL_ID,
                        get_fps(),
                        if BALL_ID > 1000 {
                            "(Press Space to Reset)"
                        } else {
                            ""
                        }
                    ),
                    15.0,
                    25.0,
                    32.0,
                    BLACK,
                );
            }
        }

        next_frame().await
    }
}
// Andrew McCall <3
// https://github.com/Andrew-McCall/MacroquadConfetti
