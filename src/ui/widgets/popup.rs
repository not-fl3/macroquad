use crate::{
    math::Vec2,
    ui::{Id, Ui},
};

/// Borderless subwindow drawn on top of everything
pub struct Popup {
    id: Id,
    size: Vec2,
}

impl Popup {
    pub fn new(id: Id, size: Vec2) -> Popup {
        Popup { id, size }
    }

    pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) {
        let pos = {
            let context = ui.get_active_window_context();
            context.window.cursor.current_position()
        };

        let _context = ui.begin_modal(self.id, pos, self.size);
        f(ui);
        ui.end_modal();
    }
}

impl Ui {
    pub fn popup<F: FnOnce(&mut Ui)>(&mut self, id: Id, size: Vec2, f: F) {
        Popup::new(id, size).ui(self, f)
    }
}
