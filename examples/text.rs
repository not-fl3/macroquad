use macroquad::prelude::*;

#[macroquad::main("Text")]
async fn main() {
    let font = load_ttf_font("./examples/DancingScriptRegular.ttf").await;

    loop {
        clear_background(BLACK);

        draw_text_ex("Custom font size:", 20.0, 20.0, TextParams::default());
        let mut y = 20.0;

        for font_size in (30..120).step_by(20) {
            let text = "abcdef";
            let params = TextParams {
                font_size,
                ..Default::default()
            };

            y += font_size as f32;
            draw_text_ex(text, 20.0, y, params);
        }

        draw_text_ex("Dynamic font scale:", 20.0, 400.0, TextParams::default());
        draw_text_ex(
            "abcd",
            20.0,
            450.0,
            TextParams {
                font_size: 50,
                font_scale: get_time().sin() as f32 / 2.0 + 1.0,
                ..Default::default()
            },
        );

        draw_text_ex("Custom font:", 400.0, 20.0, TextParams::default());
        draw_text_ex(
            "abcd",
            400.0,
            70.0,
            TextParams {
                font_size: 50,
                font,
                ..Default::default()
            },
        );

        draw_text_ex(
            "abcd",
            400.0,
            160.0,
            TextParams {
                font_size: 100,
                font,
                ..Default::default()
            },
        );

        next_frame().await
    }
}
