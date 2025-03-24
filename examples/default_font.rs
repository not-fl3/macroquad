use macroquad::prelude::*;

#[macroquad::main("DefaultFont")]
async fn main() {
    let font = load_ttf_font("./examples/DancingScriptRegular.ttf")
        .await
        .unwrap();
    set_default_font(font);

    loop {
        clear_background(WHITE);

        draw_text(
            "Hello world in a new default font!",
            100.0,
            100.0,
            40.0,
            BLACK,
        );

        draw_text_ex(
            "And with extra formatting options",
            100.0,
            230.0,
            TextParams {
                font_size: 45,
                color: RED,
                rotation: 0.27,
                ..Default::default()
            },
        );

        next_frame().await;
    }
}
