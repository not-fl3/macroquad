use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, Layout, Ui},
};

pub struct Checkbox<'a> {
    id: Id,
    label: &'a str,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: Id) -> Checkbox<'a> {
        Checkbox { id, label: "" }
    }

    pub fn label<'b>(self, label: &'b str) -> Checkbox<'b> {
        Checkbox { id: self.id, label }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut bool) {
        let context = ui.get_active_window_context();

        let label_size = context
            .window
            .painter
            .element_size(&context.style.label_style, &self.label);
        let size = vec2(
            context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
            label_size.y.max(22.),
        );

        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let whole_area = Vec2::new(
            if self.label.is_empty() {
                size.x
            } else {
                size.x / 2.0
            },
            size.y,
        );
        let checkbox_area = Vec2::new(19., 19.);
        let checkbox_pos = Vec2::new(
            pos.x + whole_area.x - 19. - 15.,
            pos.y + context.style.margin,
        );

        let hovered = Rect::new(
            checkbox_pos.x,
            checkbox_pos.y,
            checkbox_area.x,
            checkbox_area.y,
        )
        .contains(context.input.mouse_position);

        let background = context
            .style
            .checkbox_style
            .background_sprite(ElementState {
                focused: context.focused,
                hovered,
                clicked: *data,
                selected: false,
            });

        let color = context.style.checkbox_style.color(ElementState {
            focused: context.focused,
            hovered,
            clicked: hovered && context.input.is_mouse_down,
            selected: *data,
        });

        if let Some(background) = background {
            let background_margin = context
                .style
                .checkbox_style
                .background_margin
                .unwrap_or_default();

            context.window.painter.draw_sprite(
                Rect::new(checkbox_pos.x, checkbox_pos.y, 19., 19.),
                background,
                color,
                Some(background_margin),
            );
        } else {
            context.window.painter.draw_rect(
                Rect::new(
                    checkbox_pos.x,
                    checkbox_pos.y,
                    checkbox_area.x,
                    checkbox_area.y,
                ),
                None,
                color,
            );
        }

        if hovered && context.input.click_up() {
            *data ^= true;
        }

        let context = ui.get_active_window_context();

        if self.label.is_empty() == false {
            context.window.painter.draw_element_label(
                &context.style.label_style,
                Vec2::new(pos.x + size.x / 2. + 5., pos.y),
                self.label,
                ElementState {
                    focused: context.focused,
                    hovered: false,
                    clicked: false,
                    selected: false,
                },
            );
        }
    }
}

impl Ui {
    pub fn checkbox(&mut self, id: Id, label: &str, data: &mut bool) {
        Checkbox::new(id).label(label).ui(self, data)
    }
}
