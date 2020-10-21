use crate::{
    types::{Color, Vector2},
    widgets::Editbox,
    Id, Layout, Ui,
};

pub struct InputString<'a> {
    id: Id,
    label: &'a str,
    size: Option<Vector2>,
}

impl<'a> InputString<'a> {
    pub fn new(id: Id) -> InputString<'a> {
        InputString {
            id,
            size: None,
            label: "",
        }
    }

    pub fn label<'b>(self, label: &'b str) -> InputString<'b> {
        InputString {
            id: self.id,
            size: self.size,
            label,
        }
    }

    pub fn size(self, size: Vector2) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut String) {
        let context = ui.get_active_window_context();

        let size = self.size.unwrap_or_else(|| {
            Vector2::new(
                context.window.cursor.area.w
                    - context.global_style.margin * 2.
                    - context.window.cursor.ident,
                19.,
            )
        });
        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let editbox_area_w = if self.label.is_empty() {
            size.x
        } else {
            size.x / 2.
        };
        let editbox = Editbox::new(self.id, Vector2::new(editbox_area_w, size.y))
            .position(pos)
            .multiline(false);
        editbox.ui(ui, data);

        let context = ui.get_active_window_context();

        if self.label.is_empty() == false {
            context.window.draw_commands.draw_label(
                self.label,
                Vector2::new(pos.x + size.x / 2. + 5., pos.y + 2.),
                Color::from_rgba(0, 0, 0, 255),
            );
        }
    }
}

impl Ui {
    pub fn input_string(&mut self, id: Id, label: &str, data: &mut String) {
        InputString::new(id).label(label).ui(self, data)
    }
}
