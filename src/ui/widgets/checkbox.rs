use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, Layout, Ui, UiContent},
};

pub struct Checkbox<'a> {
    id: Id,
    label: &'a str,
    ratio: f32,
    pos: Option<Vec2>,
    size: Option<Vec2>,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: Id) -> Checkbox<'a> {
        Checkbox {
            id,
            label: "",
            ratio: 0.5,
            pos: None,
            size: None,
        }
    }

    pub fn ratio(self, ratio: f32) -> Self {
        Self { ratio, ..self }
    }

    pub fn label<'b>(self, label: &'b str) -> Checkbox<'b> {
        Checkbox {
            id: self.id,
            label,
            ratio: self.ratio,
            pos: self.pos,
            size: self.size,
        }
    }

    pub fn pos(self, pos: Vec2) -> Self {
        Self {
            pos: Some(pos),
            ..self
        }
    }

    pub fn size(self, size: Vec2) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut bool) {
        let context = ui.get_active_window_context();

        let label_size = context.window.painter.content_with_margins_size(
            &context.style.label_style,
            &UiContent::Label(self.label.into()),
        );
        let size = self.size.unwrap_or(vec2(
            context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
            label_size.y.max(22.),
        ));

        let pos = self
            .pos
            .unwrap_or_else(|| context.window.cursor.fit(size, Layout::Vertical));

        let whole_area = Vec2::new(
            if self.label.is_empty() {
                size.x
            } else {
                size.x * self.ratio
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
                Vec2::new(pos.x + size.x * self.ratio, pos.y),
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
