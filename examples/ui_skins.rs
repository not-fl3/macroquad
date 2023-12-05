use macroquad::prelude::*;

use macroquad::ui::{hash, root_ui, widgets, Skin};

#[macroquad::main("UI showcase")]
async fn main() {
    let skin1 = {
        let label_style = root_ui()
            .style_builder()
            .font(include_bytes!("../examples/ui_assets/HTOWERT.TTF"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 120, 255))
            .font_size(30)
            .build();

        let window_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/window_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(20.0, 20.0, 10.0, 10.0))
            .margin(RectOffset::new(-20.0, -30.0, 0.0, 0.0))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(37.0, 37.0, 5.0, 5.0))
            .margin(RectOffset::new(10.0, 10.0, 0.0, 0.0))
            .background_hovered(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_hovered_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_clicked(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_clicked_background.png"),
                    None,
                )
                .unwrap(),
            )
            .font(include_bytes!("../examples/ui_assets/HTOWERT.TTF"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 255))
            .font_size(40)
            .build();

        let editbox_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(0., 0., 0., 0.))
            .font(include_bytes!("../examples/ui_assets/HTOWERT.TTF"))
            .unwrap()
            .text_color(Color::from_rgba(120, 120, 120, 255))
            .color_selected(Color::from_rgba(190, 190, 190, 255))
            .font_size(50)
            .build();

        Skin {
            editbox_style,
            window_style,
            button_style,
            label_style,
            ..root_ui().default_skin()
        }
    };

    let skin2 = {
        let label_style = root_ui()
            .style_builder()
            .font(include_bytes!("../examples/ui_assets/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(120, 120, 120, 255))
            .font_size(25)
            .build();

        let window_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/window_background_2.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
            .margin(RectOffset::new(-30.0, 0.0, -30.0, 0.0))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_background_2.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .background_hovered(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_hovered_background_2.png"),
                    None,
                )
                .unwrap(),
            )
            .background_clicked(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/button_clicked_background_2.png"),
                    None,
                )
                .unwrap(),
            )
            .font(include_bytes!("../examples/ui_assets/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(180, 180, 100, 255))
            .font_size(40)
            .build();

        let checkbox_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/checkbox_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_hovered(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/checkbox_hovered_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_clicked(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/checkbox_clicked_background.png"),
                    None,
                )
                .unwrap(),
            )
            .build();

        let editbox_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/editbox_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(2., 2., 2., 2.))
            .font(include_bytes!("../examples/ui_assets/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(120, 120, 120, 255))
            .font_size(25)
            .build();

        let combobox_style = root_ui()
            .style_builder()
            .background(
                Image::from_file_with_format(
                    include_bytes!("../examples/ui_assets/combobox_background.png"),
                    None,
                )
                .unwrap(),
            )
            .background_margin(RectOffset::new(4., 25., 6., 6.))
            .font(include_bytes!("../examples/ui_assets/MinimalPixel v2.ttf"))
            .unwrap()
            .text_color(Color::from_rgba(120, 120, 120, 255))
            .color(Color::from_rgba(210, 210, 210, 255))
            .font_size(25)
            .build();

        Skin {
            window_style,
            button_style,
            label_style,
            checkbox_style,
            editbox_style,
            combobox_style,
            ..root_ui().default_skin()
        }
    };
    let default_skin = root_ui().default_skin().clone();

    let mut window1_skin = skin1.clone();
    let mut window2_skin = skin2.clone();

    let mut checkbox = false;
    let mut text = String::new();
    let mut number = 0.0f32;
    let mut combobox = 0;

    loop {
        clear_background(GRAY);

        root_ui().group(hash!(), vec2(70.0, 100.0), |ui| {
            ui.label(None, "Window 1");

            if ui.button(None, "Skin 1") {
                window1_skin = skin1.clone();
            }
            if ui.button(None, "Skin 2") {
                window1_skin = skin2.clone();
            }
            if ui.button(None, "No Skin") {
                window1_skin = default_skin.clone();
            }
        });
        root_ui().same_line(0.);
        root_ui().group(hash!(), vec2(70.0, 100.0), |ui| {
            ui.label(None, "Window 2");
            if ui.button(None, "Skin 1") {
                window2_skin = skin1.clone();
            }
            if ui.button(None, "Skin 2") {
                window2_skin = skin2.clone();
            }
            if ui.button(None, "No Skin") {
                window2_skin = default_skin.clone();
            }
        });

        root_ui().push_skin(&window1_skin);

        root_ui().window(hash!(), vec2(20., 250.), vec2(300., 300.), |ui| {
            widgets::Button::new("Play")
                .position(vec2(65.0, 15.0))
                .ui(ui);
            widgets::Button::new("Options")
                .position(vec2(40.0, 75.0))
                .ui(ui);

            widgets::Button::new("Quit")
                .position(vec2(65.0, 195.0))
                .ui(ui);
        });
        root_ui().pop_skin();

        root_ui().push_skin(&window2_skin);
        root_ui().window(hash!(), vec2(250., 20.), vec2(500., 250.), |ui| {
            ui.checkbox(hash!(), "Checkbox 1", &mut checkbox);
            ui.combo_box(
                hash!(),
                "Combobox",
                &["First option", "Second option"],
                &mut combobox,
            );
            ui.input_text(hash!(), "Text", &mut text);
            ui.drag(hash!(), "Drag", None, &mut number);

            widgets::Button::new("Apply")
                .position(vec2(80.0, 150.0))
                .ui(ui);
            widgets::Button::new("Cancel")
                .position(vec2(280.0, 150.0))
                .ui(ui);
        });
        root_ui().pop_skin();

        next_frame().await;
    }
}
