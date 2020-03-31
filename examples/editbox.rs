use macroquad::*;

#[macroquad::main("Input fields")]
async fn main() {
    let mut text1 = String::new();
    let mut text2 = String::new();

    loop {
        clear_background(WHITE);

        draw_window(
            hash!(),
            glam::vec2(70., 50.),
            glam::vec2(300., 300.),
            |ui| {
                ui.editbox(
                    hash!(),
                    megaui::Vector2::new(5., 0.),
                    megaui::Vector2::new(280., 280.),
                    &mut text1,
                );
            },
        );
        draw_window(
            hash!(),
            glam::vec2(300., 340.),
            glam::vec2(300., 300.),
            |ui| {
                ui.editbox(
                    hash!(),
                    megaui::Vector2::new(5., 0.),
                    megaui::Vector2::new(280., 280.),
                    &mut text2,
                );
            },
        );

        next_frame().await
    }
}
