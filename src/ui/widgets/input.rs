use crate::{
    math::{vec2, Vec2},
    ui::{widgets::Editbox, ElementState, Id, Layout, Ui, UiContent},
};

pub struct InputText<'a> {
    id: Id,
    label: &'a str,
    size: Option<Vec2>,
    password: bool,
    numbers: bool,
    ratio: f32,
    pos: Option<Vec2>,
}

impl<'a> InputText<'a> {
    pub fn new(id: Id) -> InputText<'a> {
        InputText {
            id,
            size: None,
            label: "",
            numbers: false,
            password: false,
            ratio: 0.5,
            pos: None,
        }
    }

    pub fn label<'b>(self, label: &'b str) -> InputText<'b> {
        InputText {
            id: self.id,
            size: self.size,
            label,
            numbers: self.numbers,
            password: self.password,
            ratio: self.ratio,
            pos: self.pos,
        }
    }

    pub fn size(self, size: Vec2) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

    pub fn position(self, pos: Vec2) -> Self {
        Self {
            pos: Some(pos),
            ..self
        }
    }

    pub fn password(self, password: bool) -> Self {
        Self { password, ..self }
    }

    pub fn ratio(self, ratio: f32) -> Self {
        Self { ratio, ..self }
    }

    pub fn filter_numbers(self) -> Self {
        Self {
            numbers: true,
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut String) {
        let context = ui.get_active_window_context();

        let label_size = context.window.painter.content_with_margins_size(
            &context.style.editbox_style,
            &UiContent::Label((&*data).into()),
        );

        let size = self.size.unwrap_or(vec2(
            context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
            label_size.y.max(19.),
        ));

        let pos = self
            .pos
            .unwrap_or_else(|| context.window.cursor.fit(size, Layout::Vertical));

        let editbox_area_w = if self.label.is_empty() {
            size.x
        } else {
            size.x * self.ratio - 15.
        };
        let mut editbox = Editbox::new(self.id, Vec2::new(editbox_area_w, size.y))
            .password(self.password)
            .position(pos)
            .multiline(false);

        if self.numbers {
            editbox = editbox
                .filter(&|character| character.is_digit(10) || character == '.' || character == '-')
        }
        editbox.ui(ui, data);

        let context = ui.get_active_window_context();

        if self.label.is_empty() == false {
            context.window.painter.draw_element_label(
                &context.style.label_style,
                Vec2::new(pos.x + size.x * self.ratio, pos.y),
                self.label,
                ElementState {
                    focused: context.focused,
                    ..Default::default()
                },
            );
        }
    }
}

impl Ui {
    pub fn input_text(&mut self, id: Id, label: &str, data: &mut String) {
        InputText::new(id).label(label).ui(self, data)
    }

    pub fn input_password(&mut self, id: Id, label: &str, data: &mut String) {
        InputText::new(id)
            .label(label)
            .password(true)
            .ui(self, data)
    }
}
