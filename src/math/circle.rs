use crate::math::{vec2, Rect, Vec2};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

impl Circle {
    pub fn new(x: f32, y: f32, r: f32) -> Self {
        Circle { x, y, r }
    }

    pub fn point(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    pub fn radius(&self) -> f32 {
        self.r
    }

    /// Moves the `Circle`'s origin to (x, y)
    pub fn move_to(&mut self, destination: Vec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Circle` by a factor of sr
    pub fn scale(&mut self, sr: f32) {
        self.r *= sr;
    }

    /// Checks whether the `Circle` contains a `Point`
    pub fn contains(&self, pos: &Vec2) -> bool {
        return pos.distance(vec2(self.x, self.y)) < self.r;
    }

    /// Checks whether the `Circle` overlaps a `Circle`
    pub fn overlaps(&self, other: &Circle) -> bool {
        self.point().distance(other.point()) < self.r + other.r
    }

    /// Checks whether the `Circle` overlaps a `Rect`
    pub fn overlaps_rect(&self, rect: &Rect) -> bool {
        let dist_x = (self.x - rect.center().x).abs();
        let dist_y = (self.y - rect.center().y).abs();
        if dist_x > rect.w / 2.0 + self.r || dist_y > rect.h / 2.0 + self.r {
            return false;
        }
        if dist_x <= rect.w / 2.0 || dist_y <= rect.h / 2.0 {
            return true;
        }
        let lhs = dist_x - rect.w / 2.0;
        let rhs = dist_y - rect.h / 2.0;
        let dist_sq = (lhs * lhs) + (rhs * rhs);
        return dist_sq <= self.r * self.r;
    }

    /// Translate `Circle` origin by `offset` vector
    pub fn offset(self, offset: Vec2) -> Circle {
        Circle::new(self.x + offset.x, self.y + offset.y, self.r)
    }
}
