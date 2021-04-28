use crate::{
    math::{vec2, Rect},
    ui::{widgets::Editbox, ElementState, Id, Layout, Ui},
};

use std::ops::Range;

pub struct Slider<'a> {
    id: Id,
    label: &'a str,
    range: Range<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(id: Id, range: Range<f32>) -> Slider<'a> {
        Slider {
            id,
            range,
            label: "",
        }
    }

    pub fn label<'b>(self, label: &'b str) -> Slider<'b> {
        Slider {
            id: self.id,
            range: self.range,
            label: label,
        }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut f32) {
        let context = ui.get_active_window_context();

        let size = vec2(
            context.window.cursor.area.w - context.style.margin * 3. - context.window.cursor.ident,
            19.,
        );
        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let editbox_width = 50.;
        let label_width = 100.;
        let slider_width = size.x - editbox_width - label_width;
        let margin = 5.;

        let mut temp_string = context
            .storage_any
            .get_or_insert_with::<String, _>(self.id, || format!("{:.2}", *data))
            .clone();

        let editbox_id = hash!(self.id, "editbox");
        if context.input_focused(editbox_id) == false {
            use std::fmt::Write;

            temp_string.clear();
            let _ = write!(&mut temp_string, "{:.2}", *data);
        }

        Editbox::new(editbox_id, vec2(50., size.y))
            .position(pos)
            .multiline(false)
            .filter(&|character| character.is_digit(10) || character == '.' || character == '-')
            .ui(ui, &mut temp_string);

        let context = ui.get_active_window_context();
        let old_string = context.storage_any.get_or_default::<String>(self.id);
        if *old_string != temp_string {
            if let Ok(num) = temp_string.parse::<f32>() {
                if num > self.range.end {
                    *data = self.range.end;
                } else if num < self.range.start {
                    *data = self.range.start;
                } else {
                    *data = num;
                }
            }
        }

        let dragging = context
            .storage_u32
            .entry(hash!(self.id, "dragging"))
            .or_insert(0);

        let slider_start_x = editbox_width + pos.x + margin;
        let data_pos = (*data - self.range.start) / (self.range.end - self.range.start)
            * slider_width
            + slider_start_x;

        let bar_rect = Rect::new(data_pos - 4., pos.y, 8., 20.);
        let hovered = bar_rect.contains(context.input.mouse_position);

        if hovered && context.input.is_mouse_down() {
            *dragging = 1;
            *context.input_focus = Some(self.id);
            context.input.cursor_grabbed = true;
        }

        if *dragging == 1 && context.input.is_mouse_down == false {
            context.input.cursor_grabbed = false;
            *dragging = 0;
            *context.input_focus = None;
        }

        if *dragging == 1 {
            let mouse_position = ((context.input.mouse_position.x - slider_start_x) / slider_width)
                .min(1.)
                .max(0.);
            let old_data = *data;
            *data = self.range.start + (self.range.end - self.range.start) * mouse_position;

            if old_data != *data {
                use std::fmt::Write;

                temp_string.clear();
                let _ = write!(&mut temp_string, "{:.2}", *data);
            }
        }

        // static horizontal line in the middle of the slider
        context.window.painter.draw_line(
            vec2(pos.x + editbox_width + margin, pos.y + size.y / 2.),
            vec2(
                pos.x + editbox_width + slider_width + margin,
                pos.y + size.y / 2.,
            ),
            // TODO: introduce separate slider_style
            context.style.checkbox_style.color(ElementState {
                focused: context.focused,
                hovered: true,
                clicked: true,
                selected: false,
            }),
        );

        // slider bar
        context.window.painter.draw_rect(
            bar_rect,
            None,
            context.style.checkbox_style.color(ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                selected: false,
            }),
        );

        context.window.painter.draw_element_label(
            &context.style.label_style,
            vec2(
                pos.x + editbox_width + slider_width + margin * 2.,
                pos.y + 2.,
            ),
            self.label,
            ElementState {
                focused: context.focused,
                ..Default::default()
            },
        );

        *old_string = temp_string;
    }
}

impl Ui {
    pub fn slider(&mut self, id: Id, label: &str, range: Range<f32>, data: &mut f32) {
        Slider::new(id, range).label(label).ui(self, data)
    }
}
