//! In-window drawing canvas for custom primitives like lines, rect and textures

use crate::ui::WindowContext;
use crate::Color;
use crate::Rect;
use crate::Vector2;

pub struct DrawCanvas<'a> {
    pub(crate) context: WindowContext<'a>,
}

impl<'a> DrawCanvas<'a> {
    pub fn cursor(&self) -> Vector2 {
        let cursor = &self.context.window.cursor;
        Vector2::new(cursor.x, cursor.y)
            + Vector2::new(cursor.area.x as f32, cursor.area.y as f32)
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
