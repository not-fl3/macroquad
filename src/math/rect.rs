use glam::*;

/// A 2D rectangle, defined by its top-left corner, width and height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// The x-coordinate of the top-left corner.
    pub x: f32,
    /// The y-coordinate of the top-left corner.
    pub y: f32,
    /// The width of the `Rect`, going to the right.
    pub w: f32,
    /// The height of the `Rect`, going down.
    pub h: f32,
}

impl Rect {
    /// Creates a new rectangle from its top-left corner, width and height.
    ///
    /// # Arguments:
    ///   * `x` - x-coordinate of the top-left corner.
    ///   * `y` - y-coordinate of the top-left corner.
    ///   * `w` - width of the `Rect`, going to the right.
    ///   * `h` - height of the `Rect`, going down.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    /// Returns the top-left corner of the `Rect`.
    pub const fn point(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    /// Returns the size (width and height) of the `Rect`.
    pub const fn size(&self) -> Vec2 {
        vec2(self.w, self.h)
    }

    /// Returns the center position of the `Rect`.
    pub const fn center(&self) -> Vec2 {
        vec2(self.x + self.w * 0.5f32, self.y + self.h * 0.5f32)
    }

    /// Returns the left edge of the `Rect`.
    pub const fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`.
    pub const fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`.
    pub const fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`.
    pub const fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Moves the `Rect`'s origin to (x, y).
    pub const fn move_to(&mut self, destination: Vec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left.
    pub const fn scale(&mut self, sx: f32, sy: f32) {
        self.w *= sx;
        self.h *= sy;
    }

    /// Checks whether the `Rect` contains a `Point`.
    pub const fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`.
    pub const fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub const fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect { x, y, w, h }
    }

    /// Returns an intersection rect if there is any intersection.
    pub const fn intersect(&self, other: Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(Rect {
            x: left,
            y: top,
            w: right - left,
            h: bottom - top,
        })
    }

    /// Translate rect origin by `offset` vector.
    pub const fn offset(self, offset: Vec2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectOffset {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl RectOffset {
    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> RectOffset {
        RectOffset {
            left,
            right,
            top,
            bottom,
        }
    }
}

#[test]
fn rect_contains_border() {
    let rect = Rect::new(1.0, 1.0, 2.0, 2.0);
    assert!(rect.contains(vec2(1.0, 1.0)));
    assert!(rect.contains(vec2(3.0, 1.0)));
    assert!(rect.contains(vec2(1.0, 3.0)));
    assert!(rect.contains(vec2(3.0, 3.0)));
    assert!(rect.contains(vec2(2.0, 2.0)));
}
