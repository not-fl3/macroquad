use macroquad::ui::{hash, root_ui, widgets};

use macroquad::prelude::*;

#[macroquad::main("Events")]
async fn main() {
    loop {
        clear_background(WHITE);
        root_ui().window(hash!(), Vec2::new(20., 20.), Vec2::new(450., 200.), |ui| {
            let (mouse_x, mouse_y) = mouse_position();
            ui.label(None, &format!("Mouse position: {} {}", mouse_x, mouse_y));

            let (mouse_wheel_x, mouse_wheel_y) = mouse_wheel();
            ui.label(None, &format!("Mouse wheel x: {}", mouse_wheel_x));
            ui.label(None, &format!("Mouse wheel y: {}", mouse_wheel_y));

            widgets::Group::new(hash!(), Vec2::new(200., 90.))
                .position(Vec2::new(240., 0.))
                .ui(ui, |ui| {
                    ui.label(None, "Pressed kbd keys");

                    if let Some(key) = get_last_key_pressed() {
                        ui.label(None, &format!("{:?}", key))
                    }
                });

            widgets::Group::new(hash!(), Vec2::new(200., 90.))
                .position(Vec2::new(240., 92.))
                .ui(ui, |ui| {
                    ui.label(None, "Pressed mouse keys");

                    if is_mouse_button_down(MouseButton::Left) {
                        ui.label(None, "Left");
                    }
                    if is_mouse_button_down(MouseButton::Right) {
                        ui.label(None, "Right");
                    }
                    if is_mouse_button_down(MouseButton::Middle) {
                        ui.label(None, "Middle");
                    }
                });
        });
        next_frame().await;
    }
}
