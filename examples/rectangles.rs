use macroquad::prelude::*;

#[macroquad::main("Rectangles")]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);
        let t = get_time();
        let sint = t.sin() as f32;

        draw_rectangle_ex(
            screen_width() / 2.0 - 320.0,
            100.0,
            100.0,
            100.0,
            &DrawRectangleParams {
                gradient: Some([BLUE, BLUE, PURPLE, PURPLE]),
                ..DrawRectangleParams::default()
            },
        );

        draw_rectangle_ex(
            screen_width() / 2.0 - 100.,
            100.0,
            150.0,
            190.0,
            &DrawRectangleParams {
                gradient: Some([RED, ORANGE, RED, ORANGE]),
                skew: vec2(0.5 * sint, 0.0),
                ..DrawRectangleParams::default()
            },
        );

        draw_rectangle_ex(
            screen_width() / 2.0 + 150.0,
            100.0,
            120.0,
            60.0,
            &DrawRectangleParams {
                gradient: Some([SKYBLUE, WHITE, GREEN, DARKPURPLE]),
                skew: vec2(0.8 * sint, 0.8 * sint),
                border_radius: 10.0,
                ..DrawRectangleParams::default()
            },
        );

        draw_rectangle_ex(
            screen_width() / 2.0 - 320.0,
            300.0,
            150.0,
            90.0,
            &DrawRectangleParams {
                rotation: t as f32,
                gradient: Some([ORANGE, ORANGE, YELLOW, YELLOW]),
                border_radius: 20.0 + sint * 20.,
                line_thickness: 5.0,
                ..DrawRectangleParams::default()
            },
        );

        draw_rectangle_ex(
            screen_width() / 2.0 - 320.0,
            300.0,
            180.0,
            180.0,
            &DrawRectangleParams {
                rotation: t as f32,
                pivot: Some(vec2(screen_width() / 2.0, screen_height() / 2.0)),
                gradient: Some([BLUE, PURPLE, PURPLE, BLUE]),
                border_radius: 40.0,
                border_radius_segments: 10,
                line_thickness: 16.0 + sint * 15.,
                ..DrawRectangleParams::default()
            },
        );
        next_frame().await
    }
}
