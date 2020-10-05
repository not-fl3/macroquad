use macroquad::prelude::*;

#[macroquad::main("TestWindow")]
async fn main() {
    loop {
        clear_background(RED);

        draw_window(
            hash!(),
            Vec2::new(20., 20.),
            Vec2::new(200., 200.),
            None,
            |ui| {
                for i in 0..30 {
                    ui.tree_node(hash!(i), &format!("Node {}", i), |ui| {
                        ui.label(None, "TEXT");
                        ui.label(None, "MORE TEXT");
                        ui.button(None, "OK?");
                    });
                }
            },
        );

        draw_window(
            hash!(),
            Vec2::new(220., 70.),
            Vec2::new(200., 200.),
            None,
            |ui| {
                ui.label(None, "TEXT");
                ui.button(None, "OK?");
            },
        );

        next_frame().await
    }
}
