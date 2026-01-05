use macroquad::prelude::*;

use std::collections::LinkedList;

const SQUARES: i16 = 16;

type Point = (i16, i16);

struct Snake {
    head: Point,
    body: LinkedList<Point>,
    dir: Point,
}

#[macroquad::main("Snake")]
async fn main() {
    let mut snake = Snake {
        head: (0, 0),
        dir: (1, 0),
        body: LinkedList::new(),
    };
    let mut fruit: Point = (rand::gen_range(0, SQUARES), rand::gen_range(0, SQUARES));
    let mut score = 0;
    let mut speed = 0.3;
    let mut last_update = get_time();
    // A queue to buffer player inputs for more responsive controls
    let mut input_buffer: LinkedList<Point> = LinkedList::new();
    let mut game_over = false;

    let up = (0, -1);
    let down = (0, 1);
    let right = (1, 0);
    let left = (-1, 0);

    loop {
        if !game_over {
            // Get the last direction command given, or the snake's current direction if buffer is empty.
            // This prevents adding opposite moves to the buffer.
            let last_dir = input_buffer.back().cloned().unwrap_or(snake.dir);

            // Use is_key_pressed to register input only once per press
            if is_key_pressed(KeyCode::Right) && last_dir != left {
                input_buffer.push_back(right);
            } else if is_key_pressed(KeyCode::Left) && last_dir != right {
                input_buffer.push_back(left);
            } else if is_key_pressed(KeyCode::Up) && last_dir != down {
                input_buffer.push_back(up);
            } else if is_key_pressed(KeyCode::Down) && last_dir != up {
                input_buffer.push_back(down);
            }

            if get_time() - last_update > speed {
                last_update = get_time();

                // Process one command from the input buffer each tick
                if let Some(new_dir) = input_buffer.pop_front() {
                    // Extra check to ensure the snake doesn't reverse on itself from a buffered command
                    if new_dir.0 != -snake.dir.0 || new_dir.1 != -snake.dir.1 {
                        snake.dir = new_dir;
                    }
                }

                snake.body.push_front(snake.head);
                snake.head = (snake.head.0 + snake.dir.0, snake.head.1 + snake.dir.1);
                if snake.head == fruit {
                    fruit = (rand::gen_range(0, SQUARES), rand::gen_range(0, SQUARES));
                    score += 100;
                    speed *= 0.9;
                } else {
                    snake.body.pop_back();
                }
                if snake.head.0 < 0
                    || snake.head.1 < 0
                    || snake.head.0 >= SQUARES
                    || snake.head.1 >= SQUARES
                {
                    game_over = true;
                }
                for (x, y) in &snake.body {
                    if *x == snake.head.0 && *y == snake.head.1 {
                        game_over = true;
                    }
                }
            }
        }
        if !game_over {
            clear_background(LIGHTGRAY);

            let game_size = screen_width().min(screen_height());
            let offset_x = (screen_width() - game_size) / 2. + 10.;
            let offset_y = (screen_height() - game_size) / 2. + 10.;
            let sq_size = (screen_height() - offset_y * 2.) / SQUARES as f32;

            draw_rectangle(offset_x, offset_y, game_size - 20., game_size - 20., WHITE);

            for i in 1..SQUARES {
                draw_line(
                    offset_x,
                    offset_y + sq_size * i as f32,
                    screen_width() - offset_x,
                    offset_y + sq_size * i as f32,
                    2.,
                    LIGHTGRAY,
                );
            }

            for i in 1..SQUARES {
                draw_line(
                    offset_x + sq_size * i as f32,
                    offset_y,
                    offset_x + sq_size * i as f32,
                    screen_height() - offset_y,
                    2.,
                    LIGHTGRAY,
                );
            }

            draw_rectangle(
                offset_x + snake.head.0 as f32 * sq_size,
                offset_y + snake.head.1 as f32 * sq_size,
                sq_size,
                sq_size,
                DARKGREEN,
            );

            for (x, y) in &snake.body {
                draw_rectangle(
                    offset_x + *x as f32 * sq_size,
                    offset_y + *y as f32 * sq_size,
                    sq_size,
                    sq_size,
                    LIME,
                );
            }

            draw_rectangle(
                offset_x + fruit.0 as f32 * sq_size,
                offset_y + fruit.1 as f32 * sq_size,
                sq_size,
                sq_size,
                GOLD,
            );

            draw_text(format!("SCORE: {score}").as_str(), 10., 20., 20., DARKGRAY);
        } else {
            clear_background(WHITE);
            let text = "Game Over. Press [enter] to play again.";
            let font_size = 30.;
            let text_size = measure_text(text, None, font_size as _, 1.0);

            draw_text(
                text,
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. + text_size.height / 2.,
                font_size,
                DARKGRAY,
            );

            if is_key_down(KeyCode::Enter) {
                snake = Snake {
                    head: (0, 0),
                    dir: (1, 0),
                    body: LinkedList::new(),
                };
                fruit = (rand::gen_range(0, SQUARES), rand::gen_range(0, SQUARES));
                score = 0;
                speed = 0.3;
                last_update = get_time();
                input_buffer.clear(); // Clear buffer on restart
                game_over = false;
            }
        }
        next_frame().await;
    }
}
