use crate::{Layout, Ui};
use glam::Vec2;

impl Ui {
    pub fn separator(&mut self) {
        let context = self.get_active_window_context();

        let size = Vec2::new(
            context.window.cursor.area.w
                - context.global_style.margin * 2.
                - context.window.cursor.ident,
            5.,
        );

        let pos = context.window.cursor.fit(size, Layout::Vertical);

        context.window.draw_commands.draw_line(
            Vec2::new(pos.x(), pos.y() + 2.),
            Vec2::new(pos.x() + size.x(), pos.y() + 2.),
            context.global_style.separator(context.focused),
        );
    }
}
