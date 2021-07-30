use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets::Window};

#[macroquad::main("Exit dialog")]
async fn main() {
    prevent_quit();

    let mut show_exit_dialog = false;
    let mut user_decided_to_exit = false;

    loop {
        clear_background(GRAY);

        if is_quit_requested() {
            show_exit_dialog = true;
        }

        if show_exit_dialog {
            let dialog_size = vec2(200., 70.);
            let screen_size = vec2(screen_width(), screen_height());
            let dialog_position = screen_size / 2. - dialog_size / 2.;
            Window::new(hash!(), dialog_position, dialog_size).ui(&mut *root_ui(), |ui| {
                ui.label(None, "Do you really want to quit?");
                ui.separator();
                ui.same_line(60.);
                if ui.button(None, "Yes") {
                    user_decided_to_exit = true;
                }
                ui.same_line(120.);
                if ui.button(None, "No") {
                    show_exit_dialog = false;
                }
            });
        }

        if user_decided_to_exit {
            break;
        }

        next_frame().await
    }
}
