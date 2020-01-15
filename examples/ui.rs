use macroquad::*;

use macroquad::hash;

fn main() {
    Window::init("BasicShapes").main_loop(|| {
        draw_window(hash!(), Vec2::new(20., 20.), Vec2::new(200., 200.), |ui| {
            for i in 0..30 {
                ui.tree_node(hash!(i), &format!("Node {}", i), |ui| {
                    ui.label(None, "TEXT");
                    ui.label(None, "MORE TEXT");
                    ui.button(None, "OK?");
                });
            }
        });

        draw_window(hash!(), Vec2::new(220., 70.), Vec2::new(200., 200.), |ui| {
            ui.label(None, "TEXT");
            ui.button(None, "OK?");
        });
    });
}
