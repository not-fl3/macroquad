use crate::{
    math::Vec2,
    ui::{Layout, Ui},
};

impl Ui {
    pub fn separator(&mut self) {
        let context = self.get_active_window_context();

        // hack: to move cursor to the beginning of next line
        context
            .window
            .cursor
            .fit(Vec2::new(0., 1.), Layout::Vertical);

        // let _size = Vec2::new(
        //     context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
        //     5.,
        // );
        // let pos = context.window.cursor.fit(size, Layout::Vertical);
        // context.window.painter.draw_line(
        //     Vec2::new(pos.x, pos.y + 2.),
        //     Vec2::new(pos.x + size.x, pos.y + 2.),
        //     context.style.separator(context.focused),
        // );
    }
}
