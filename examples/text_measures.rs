use macroquad::prelude::*;

fn draw_text_annotated(text: &str, font: Option<&Font>, x: f32, baseline: f32) {
    let size = measure_text(text, font, 100, 1.0);

    // Full background rect
    draw_rectangle(x, baseline - size.offset_y, size.width, size.height, BLUE);

    // Base line
    draw_rectangle(x, baseline - 2.0, size.width, 4.0, RED);

    // Base line annotation
    draw_rectangle(x + size.width, baseline - 1.0, 120.0, 1.0, GRAY);
    draw_text(
        "baseline",
        x + size.width + 10.0,
        baseline - 5.0,
        30.0,
        WHITE,
    );

    // Top line
    draw_rectangle(x, baseline - 2.0 - size.offset_y, size.width, 4.0, RED);

    // Top line annotation
    draw_rectangle(
        x + size.width,
        baseline - size.offset_y - 1.0,
        120.0,
        1.0,
        GRAY,
    );
    draw_text(
        "topline",
        x + size.width + 10.0,
        baseline - size.offset_y - 5.0,
        30.0,
        WHITE,
    );

    // Bottom line
    draw_rectangle(
        x,
        baseline - 2.0 - size.offset_y + size.height,
        size.width,
        4.0,
        RED,
    );

    // Bottom line annotation
    draw_rectangle(
        x + size.width,
        baseline - size.offset_y + size.height - 1.0,
        120.0,
        1.0,
        GRAY,
    );
    draw_text(
        "bottomline",
        x + size.width + 10.0,
        baseline - size.offset_y + size.height - 5.0,
        30.0,
        WHITE,
    );

    draw_text_ex(
        text,
        x,
        baseline,
        TextParams {
            font_size: 100,
            font,
            ..Default::default()
        },
    );
}

#[macroquad::main("Text")]
async fn main() {
    let font = load_ttf_font("./examples/DancingScriptRegular.ttf")
        .await
        .unwrap();

    loop {
        clear_background(BLACK);

        let text = "abcdIj";

        draw_text_annotated(text, None, 40.0, 200.0);
        draw_text_annotated(text, Some(&font), 400.0, 400.0);

        next_frame().await
    }
}
