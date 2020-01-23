use macroquad::*;

fn main() {
    const BLOCKS_W: usize = 10;
    const BLOCKS_H: usize = 10;
    const SCR_W: f32 = 20.0;
    const SCR_H: f32 = 20.0;

    let mut blocks: [[bool; BLOCKS_W]; BLOCKS_H] = [[true; BLOCKS_W]; BLOCKS_H];
    let mut ball_x = 12.;
    let mut ball_y = 7.;
    let mut dx = 0.15;
    let mut dy = -0.15;
    let mut platform_x = 10.;
    let mut stick = true;
    let platform_width = 5.;
    let platform_height = 0.2;

    Window::new("Input")
        .on_init(|| {
            set_screen_coordinates(ScreenCoordinates::Fixed(0., SCR_W, SCR_H, 0.));
        })
        .main_loop(|| {
            clear_background(SKYBLUE);

            if is_key_down(KeyCode::Right) && platform_x < SCR_W - platform_width / 2. {
                platform_x += 1.0;
            }
            if is_key_down(KeyCode::Left) && platform_x > platform_width / 2. {
                platform_x -= 1.0;
            }

            if stick == false {
                ball_x += dx;
                ball_y += dy;
            } else {
                draw_text("Press space to start", 5., 10., 0.5, BLACK);

                ball_x = platform_x;
                ball_y = SCR_H - 0.5;

                stick = !is_key_down(KeyCode::Space);
            }

            if ball_x <= 0. || ball_x > SCR_W {
                dx *= -1.;
            }
            if ball_y <= 0.
                || (ball_y > SCR_H - platform_height - 0.15 / 2.
                    && ball_x >= platform_x - platform_width / 2.
                    && ball_x <= platform_x + platform_width / 2.)
            {
                dy *= -1.;
            }
            if ball_y >= SCR_H {
                ball_y = 10.;
                dy = -dy.abs();
                stick = true;
            }

            for j in 0..BLOCKS_H {
                for i in 0..BLOCKS_W {
                    if blocks[j][i] {
                        let block_w = SCR_W / BLOCKS_W as f32;
                        let block_h = 7.0 / BLOCKS_H as f32;
                        let block_x = i as f32 * block_w + 0.05;
                        let block_y = j as f32 * block_h + 0.05;

                        draw_rectangle(block_x, block_y, block_w - 0.1, block_h - 0.1, MAROON);

                        if ball_x >= block_x
                            && ball_x < block_x + block_w
                            && ball_y >= block_y
                            && ball_y < block_y + block_h
                        {
                            dy *= -1.;
                            blocks[j][i] = false;
                        }
                    }
                }
            }

            draw_circle(ball_x, ball_y, 0.15, GREEN);
            draw_rectangle(
                platform_x - platform_width / 2.,
                SCR_H - platform_height,
                platform_width,
                platform_height,
                DARKBLUE,
            );
        });
}
