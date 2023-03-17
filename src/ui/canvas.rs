//! In-window drawing canvas for custom primitives like lines, rect and textures

use super::Layout;
use super::WindowContext;
use crate::{
    color::Color,
    math::{Rect, Vec2},
    texture::Texture2D,
};

pub struct DrawCanvas<'a> {
    pub(crate) context: WindowContext<'a>,
}

impl<'a> DrawCanvas<'a> {
    pub fn cursor(&self) -> Vec2 {
        let cursor = &self.context.window.cursor;
        Vec2::new(cursor.x, cursor.y)
            + Vec2::new(cursor.area.x as f32, cursor.area.y as f32)
            + cursor.scroll.scroll
    }

    pub fn request_space(&mut self, space: Vec2) -> Vec2 {
        let cursor = &mut self.context.window.cursor;

        cursor.fit(space, Layout::Vertical)
    }

    pub fn rect<S, T>(&mut self, rect: Rect, stroke: S, fill: T)
    where
        S: Into<Option<Color>>,
        T: Into<Option<Color>>,
    {
        self.context.register_click_intention(rect);

        self.context.window.painter.draw_rect(rect, stroke, fill);
    }

    pub fn line(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.context.window.painter.draw_line(start, end, color);
    }

    pub fn image(&mut self, rect: Rect, texture: &Texture2D) {
        self.context.register_click_intention(rect);

        self.context.window.painter.draw_raw_texture(rect, texture);
    }
}
