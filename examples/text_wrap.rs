use macroquad::prelude::*;

static LOREM: &str = "Lorem ipsum odor amet, consectetuer adipiscing elit. Ultrices nostra volutpat facilisis magna mus. Rhoncus tempor feugiat netus maecenas pretium leo vitae. Eros aliquet maecenas eu diam aliquet varius hac elementum. Sociosqu platea per ultricies vitae praesent mauris nostra ridiculus. Est cursus pulvinar efficitur mus vel leo. Integer et nec eleifend non leo. Lorem rutrum ultrices potenti facilisis hendrerit facilisi metus sit. AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

Intentional newlines
are preserved.";

#[macroquad::main("Text Wrap")]
async fn main() {
    let font_size = 24;
    loop {
        clear_background(BLACK);

        let maximum_line_length = f32::max(20.0, mouse_position().0 - 20.0);
        let text = wrap_text(LOREM, None, font_size, 1.0, maximum_line_length);
        let dimensions = measure_multiline_text(&text, None, font_size, 1.0, Some(1.0));

        draw_multiline_text(
            &text,
            20.0,
            20.0 + dimensions.offset_y,
            font_size as f32,
            Some(1.0),
            WHITE,
        );
        draw_rectangle_lines(20.0, 20.0, dimensions.width, dimensions.height, 2.0, BLUE);
        draw_line(
            20.0 + maximum_line_length,
            0.0,
            20.0 + maximum_line_length,
            screen_height(),
            1.0,
            RED,
        );

        next_frame().await
    }
}
