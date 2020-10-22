//! In-window drawing canvas for custom primitives like lines, rect and textures

use crate::ui::WindowContext;
use crate::Color;
use crate::Rect;
use glam::Vec2;

pub struct DrawCanvas<'a> {
    pub(crate) context: WindowContext<'a>,
}

impl<'a> DrawCanvas<'a> {
    pub fn cursor(&self) -> Vec2 {
        let cursor = &self.context.window.cursor;
        Vec2::new(cursor.x, cursor.y)
            + Vec2::new(cursor.area.x, cursor.area.y)
            + cursor.scroll.scroll
    }

    pub fn rect<S, T>(&mut self, rect: Rect, stroke: S, fill: T)
    where
        S: Into<Option<Color>>,
        T: Into<Option<Color>>,
    {
        self.context
            .window
            .draw_commands
            .draw_rect(rect, stroke, fill);
    }

    pub fn image(&mut self, rect: Rect, texture: u32) {
            self.context
            .window
            .draw_commands
            .draw_raw_texture(rect, texture);
    }
}
