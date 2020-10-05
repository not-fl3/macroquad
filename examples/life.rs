use macroquad::prelude::*;

#[macroquad::main("Life")]
async fn main() {
    let w = screen_width() as usize;
    let h = screen_height() as usize;

    let mut buffer = vec![WHITE; w * h];
    let mut image = Image::gen_image_color(w as u16, h as u16, WHITE);
    let image_data = image.get_image_data_mut();
    for y in 0..h {
        for x in 0..w {
            if rand::gen_range(0, 5) == 0 {
                image_data[y * w + x] = BLACK;
            }
        }
    }
    let texture = load_texture_from_image(&image);

    loop {
        clear_background(WHITE);

        let w = image.width();
        let h = image.height();

        let image_data = image.get_image_data();
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut neighbors_count = 0;

                for j in -1i32..=1 {
                    for i in -1i32..=1 {
                        // out of bounds
                        if y + j < 0 || y + j >= h as i32 || x + i < 0 || x + i >= w as i32 {
                            continue;
                        }
                        // cell itself
                        if i == 0 && j == 0 {
                            continue;
                        }

                        let neighbor = image_data[(y + j) as usize * w + (x + i) as usize];
                        if neighbor == BLACK {
                            neighbors_count += 1;
                        }
                    }
                }

                let current_cell = image_data[y as usize * w + x as usize];
                buffer[y as usize * w + x as usize] = match (current_cell, neighbors_count) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (BLACK, x) if x < 2 => WHITE,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (BLACK, 2) | (BLACK, 3) => BLACK,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (BLACK, x) if x > 3 => WHITE,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (WHITE, 3) => BLACK,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };
            }
        }

        image.update(&buffer);

        update_texture(texture, &image);

        draw_texture(texture, 0., 0., WHITE);

        next_frame().await
    }
}
