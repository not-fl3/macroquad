//! Several methods in megaui, such as `Ui::scroll_here`, implicitly rely on the concept of a GUI cursor.
//! This gui cursor is not to be confused with the mouse cursor.
//! Instead it describes where the next widget will be placed
//! if you do not explicitly set its position with Layout::Free.

use crate::math::{Rect, Vec2};

#[derive(Clone, Debug)]
pub struct Scroll {
    pub scroll: Vec2,
    pub dragging_x: bool,
    pub dragging_y: bool,
    pub rect: Rect,
    pub inner_rect: Rect,
    pub inner_rect_previous_frame: Rect,
    pub initial_scroll: Vec2,
}

impl Scroll {
    pub fn scroll_to(&mut self, y: f32) {
        self.rect.y = y
            .max(self.inner_rect_previous_frame.y)
            .min(self.inner_rect_previous_frame.h - self.rect.h + self.inner_rect_previous_frame.y);
    }

    pub fn update(&mut self) {
        self.rect.y =
            self.rect.y.max(self.inner_rect_previous_frame.y).min(
                self.inner_rect_previous_frame.h - self.rect.h + self.inner_rect_previous_frame.y,
            );
    }
}

#[derive(Debug, Clone)]
pub enum Layout {
    Vertical,
    Horizontal,
    Free(Vec2),
}

#[derive(Debug)]
pub struct Cursor {
    pub x: f32,
    pub y: f32,
    pub start_x: f32,
    pub start_y: f32,
    pub ident: f32,
    pub scroll: Scroll,
    pub area: Rect,
    pub margin: f32,
    pub next_same_line: Option<f32>,
    pub max_row_y: f32,
}

impl Cursor {
    pub fn new(area: Rect, margin: f32) -> Cursor {
        Cursor {
            margin,
            x: margin,
            y: margin,
            ident: 0.,
            start_x: margin,
            start_y: margin,
            scroll: Scroll {
                rect: Rect::new(0., 0., area.w, area.h),
                inner_rect: Rect::new(0., 0., area.w, area.h),
                inner_rect_previous_frame: Rect::new(0., 0., area.w, area.h),
                scroll: Vec2::new(0., 0.),
                dragging_x: false,
                dragging_y: false,
                initial_scroll: Vec2::new(0., 0.),
            },
            area,
            next_same_line: None,
            max_row_y: 0.,
        }
    }

    pub fn reset(&mut self) {
        self.x = self.start_x;
        self.y = self.start_y;
        self.max_row_y = 0.;
        self.ident = 0.;
        self.scroll.inner_rect_previous_frame = self.scroll.inner_rect;
        self.scroll.inner_rect = Rect::new(0., 0., self.area.w, self.area.h);
    }

    pub fn current_position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
            + Vec2::new(self.area.x as f32, self.area.y as f32)
            + self.scroll.scroll
            + Vec2::new(self.ident, 0.)
    }

    pub fn fit(&mut self, size: Vec2, mut layout: Layout) -> Vec2 {
        let res;

        if let Some(x) = self.next_same_line {
            self.next_same_line = None;
            if x != 0.0 {
                self.x = x;
            }
            layout = Layout::Horizontal;
        }
        match layout {
            Layout::Horizontal => {
                self.max_row_y = self.max_row_y.max(size.y);

                if self.x + size.x < self.area.w as f32 - self.margin * 2. {
                    res = Vec2::new(self.x, self.y);
                } else {
                    self.x = self.margin + 1.; // +1. is a hack to make next vertical thing correctly jump to the next row
                    self.y += self.max_row_y + self.margin;
                    self.max_row_y = 0.;
                    res = Vec2::new(self.x, self.y);
                }
                self.x += size.x + self.margin;
            }
            Layout::Vertical => {
                if self.x != self.margin {
                    self.x = self.margin;
                    self.y += self.max_row_y;
                }
                res = Vec2::new(self.x, self.y);
                self.x += size.x + self.margin;
                self.max_row_y = size.y + self.margin;
            }
            Layout::Free(point) => {
                res = point;
            }
        }
        self.scroll.inner_rect = self
            .scroll
            .inner_rect
            .combine_with(Rect::new(res.x, res.y, size.x, size.y));

        res + Vec2::new(self.area.x as f32, self.area.y as f32)
            + self.scroll.scroll
            + Vec2::new(self.ident, 0.)
    }
}
