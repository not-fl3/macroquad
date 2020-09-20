use crate::{
    types::{Color, Vector2},
    widgets::Editbox,
    Id, Layout, Ui,
};

pub struct InputField<'a> {
    id: Id,
    label: &'a str,
}

impl<'a> InputField<'a> {
    pub fn new(id: Id) -> InputField<'a> {
        InputField { id, label: "" }
    }

    pub fn label<'b>(self, label: &'b str) -> InputField<'b> {
        InputField { id: self.id, label }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut String) {
        let context = ui.get_active_window_context();

        let size = Vector2::new(
            context.window.cursor.area.w
                - context.global_style.margin * 2.
                - context.window.cursor.ident,
            19.,
        );
        let pos = context.window.cursor.fit(size, Layout::Vertical);

        Editbox::new(self.id, Vector2::new(size.x / 2., size.y))
            .position(pos)
            .multiline(false)
            .ui(ui, data);

        let context = ui.get_active_window_context();

        context.window.draw_commands.draw_label(
            self.label,
            Vector2::new(pos.x + size.x / 2. + 5., pos.y + 2.),
            Color::from_rgba(0, 0, 0, 255),
        );
    }
}

impl Ui {
    pub fn input_field(&mut self, id: Id, label: &str, data: &mut String) {
        InputField::new(id).label(label).ui(self, data)
    }
}
