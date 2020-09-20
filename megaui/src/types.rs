#[derive(Copy, Clone, Default, Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn distance(self, other: Vector2) -> f32 {
        ((self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)).sqrt()
    }
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// A simple 2D rectangle.
///
/// The origin of the rectangle is at the top-left,
/// with x increasing to the right and y increasing down.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Rect {
    /// X coordinate of the left edge of the rect.
    pub x: f32,
    /// Y coordinate of the top edge of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new `Rect`.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    /// Creates a new `Rect` a la Love2D's `love.graphics.newQuad`,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

    /// Create a new rect from `i32` coordinates.
    pub fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Create a new `Rect` with all values zero.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new `Rect` at `0,0` with width and height 1.
    pub fn one() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains(&self, point: Vector2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect { x, y, w, h }
    }

    pub fn intersect(&self, other: Rect) -> Option<Rect> {
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

    pub fn offset(self, offset: Vector2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }
}

/// A RGBA color in the `sRGB` color space represented as `f32`'s in the range `[0.0-1.0]`
///
/// For convenience, [`WHITE`](constant.WHITE.html) and [`BLACK`](constant.BLACK.html) are provided.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

impl Color {
    /// Create a new `Color` from four `f32`'s in the range `[0.0-1.0]`
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new `Color` from four `u8`'s in the range `[0-255]`
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::from((r, g, b, a))
    }

    /// Create a new `Color` from three u8's in the range `[0-255]`,
    /// with the alpha component fixed to 255 (opaque)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from((r, g, b))
    }

    /// Return a tuple of four `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgba(self) -> (u8, u8, u8, u8) {
        self.into()
    }

    /// Return a tuple of three `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        self.into()
    }

    /// Convert a packed `u32` containing `0xRRGGBBAA` into a `Color`
    pub fn from_rgba_u32(c: u32) -> Color {
        let rp = ((c & 0xFF00_0000u32) >> 24) as u8;
        let gp = ((c & 0x00FF_0000u32) >> 16) as u8;
        let bp = ((c & 0x0000_FF00u32) >> 8) as u8;
        let ap = (c & 0x0000_00FFu32) as u8;
        Color::from((rp, gp, bp, ap))
    }

    /// Convert a packed `u32` containing `0x00RRGGBB` into a `Color`.
    /// This lets you do things like `Color::from_rgb_u32(0xCD09AA)` easily if you want.
    pub fn from_rgb_u32(c: u32) -> Color {
        let rp = ((c & 0x00FF_0000u32) >> 16) as u8;
        let gp = ((c & 0x0000_FF00u32) >> 8) as u8;
        let bp = (c & 0x0000_00FFu32) as u8;
        Color::from((rp, gp, bp))
    }

    /// Convert a `Color` into a packed `u32`, containing `0xRRGGBBAA` as bytes.
    pub fn to_rgba_u32(self) -> u32 {
        let (r, g, b, a): (u8, u8, u8, u8) = self.into();
        let rp = (u32::from(r)) << 24;
        let gp = (u32::from(g)) << 16;
        let bp = (u32::from(b)) << 8;
        let ap = u32::from(a);
        rp | gp | bp | ap
    }

    /// Convert a `Color` into a packed `u32`, containing `0x00RRGGBB` as bytes.
    pub fn to_rgb_u32(self) -> u32 {
        let (r, g, b, _a): (u8, u8, u8, u8) = self.into();
        let rp = (u32::from(r)) << 16;
        let gp = (u32::from(g)) << 8;
        let bp = u32::from(b);
        rp | gp | bp
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `u8`'s in the range `[0-255]` into a `Color`
    fn from(val: (u8, u8, u8, u8)) -> Self {
        let (r, g, b, a) = val;
        let rf = (f32::from(r)) / 255.0;
        let gf = (f32::from(g)) / 255.0;
        let bf = (f32::from(b)) / 255.0;
        let af = (f32::from(a)) / 255.0;
        Color::new(rf, gf, bf, af)
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Convert a `(R, G, B)` tuple of `u8`'s in the range `[0-255]` into a `Color`,
    /// with a value of 255 for the alpha element (i.e., no transparency.)
    fn from(val: (u8, u8, u8)) -> Self {
        let (r, g, b) = val;
        Color::from((r, g, b, 255))
    }
}

impl From<[f32; 4]> for Color {
    /// Turns an `[R, G, B, A] array of `f32`'s into a `Color` with no format changes.
    /// All inputs should be in the range `[0.0-1.0]`.
    fn from(val: [f32; 4]) -> Self {
        Color::new(val[0], val[1], val[2], val[3])
    }
}

impl From<(f32, f32, f32)> for Color {
    /// Convert a `(R, G, B)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`,
    /// with a value of 1.0 to for the alpha element (ie, no transparency.)
    fn from(val: (f32, f32, f32)) -> Self {
        let (r, g, b) = val;
        Color::new(r, g, b, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`
    fn from(val: (f32, f32, f32, f32)) -> Self {
        let (r, g, b, a) = val;
        Color::new(r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B, A)` tuple of `u8`'s in the range of `[0-255]`.
    fn from(color: Color) -> Self {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        (r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B)` tuple of `u8`'s in the range of `[0-255]`,
    /// ignoring the alpha term.
    fn from(color: Color) -> Self {
        let (r, g, b, _) = color.into();
        (r, g, b)
    }
}

impl From<Color> for [f32; 4] {
    /// Convert a `Color` into an `[R, G, B, A]` array of `f32`'s in the range of `[0.0-1.0]`.
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<(i32, i32, i32, f32)> for Color {
    fn from((r, g, b, a): (i32, i32, i32, f32)) -> Color {
        Color::new(r as f32 / 256., g as f32 / 256., b as f32 / 256., a)
    }
}

impl Into<String> for Color {
    fn into(self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            (self.r * 255.) as i32,
            (self.g * 255.) as i32,
            (self.b * 255.) as i32,
            (self.a * 255.) as i32
        )
    }
}
