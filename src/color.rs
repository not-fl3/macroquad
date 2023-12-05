//! Color types and helpers.

pub use colors::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Build a color from 4 components of 0..255 values
/// This is a temporary solution and going to be replaced with const fn,
/// waiting for [this issue](https://github.com/rust-lang/rust/issues/57241) to be resolved.
#[macro_export]
macro_rules! color_u8 {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        Color::new(
            $r as f32 / 255.,
            $g as f32 / 255.,
            $b as f32 / 255.,
            $a as f32 / 255.,
        )
    };
}

#[test]
fn color_from_bytes() {
    assert_eq!(Color::new(1.0, 0.0, 0.0, 1.0), color_u8!(255, 0, 0, 255));
    assert_eq!(
        Color::new(1.0, 0.5, 0.0, 1.0),
        color_u8!(255, 127.5, 0, 255)
    );
    assert_eq!(
        Color::new(0.0, 1.0, 0.5, 1.0),
        color_u8!(0, 255, 127.5, 255)
    );
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        [
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        ]
    }
}

impl Into<Color> for [u8; 4] {
    fn into(self) -> Color {
        Color::new(
            self[0] as f32 / 255.,
            self[1] as f32 / 255.,
            self[2] as f32 / 255.,
            self[3] as f32 / 255.,
        )
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(colors: [f32; 4]) -> Color {
        Color::new(colors[0], colors[1], colors[2], colors[3])
    }
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// Build a color from 4 0..255 components.
    /// Unfortunately it can't be const fn due to [this issue](https://github.com/rust-lang/rust/issues/57241).
    /// When const version is needed "color_u8" macro may be a workaround.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::new(
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        )
    }

    /// Build a color from a hexadecimal u32
    /// Example: 0x3CA7D5 - a light blue
    pub fn from_hex(hex: u32) -> Color {
        let bytes: [u8; 4] = hex.to_be_bytes();

        Self::from_rgba(bytes[1], bytes[2], bytes[3], 255)
    }

    pub fn to_vec(&self) -> glam::Vec4 {
        glam::Vec4::new(self.r, self.g, self.b, self.a)
    }

    pub fn from_vec(vec: glam::Vec4) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }
}

pub mod colors {
    //! Constants for some common colors.

    use super::Color;

    pub const LIGHTGRAY: Color = Color::new(0.78, 0.78, 0.78, 1.00);
    pub const GRAY: Color = Color::new(0.51, 0.51, 0.51, 1.00);
    pub const DARKGRAY: Color = Color::new(0.31, 0.31, 0.31, 1.00);
    pub const YELLOW: Color = Color::new(0.99, 0.98, 0.00, 1.00);
    pub const GOLD: Color = Color::new(1.00, 0.80, 0.00, 1.00);
    pub const ORANGE: Color = Color::new(1.00, 0.63, 0.00, 1.00);
    pub const PINK: Color = Color::new(1.00, 0.43, 0.76, 1.00);
    pub const RED: Color = Color::new(0.90, 0.16, 0.22, 1.00);
    pub const MAROON: Color = Color::new(0.75, 0.13, 0.22, 1.00);
    pub const GREEN: Color = Color::new(0.00, 0.89, 0.19, 1.00);
    pub const LIME: Color = Color::new(0.00, 0.62, 0.18, 1.00);
    pub const DARKGREEN: Color = Color::new(0.00, 0.46, 0.17, 1.00);
    pub const SKYBLUE: Color = Color::new(0.40, 0.75, 1.00, 1.00);
    pub const BLUE: Color = Color::new(0.00, 0.47, 0.95, 1.00);
    pub const DARKBLUE: Color = Color::new(0.00, 0.32, 0.67, 1.00);
    pub const PURPLE: Color = Color::new(0.78, 0.48, 1.00, 1.00);
    pub const VIOLET: Color = Color::new(0.53, 0.24, 0.75, 1.00);
    pub const DARKPURPLE: Color = Color::new(0.44, 0.12, 0.49, 1.00);
    pub const BEIGE: Color = Color::new(0.83, 0.69, 0.51, 1.00);
    pub const BROWN: Color = Color::new(0.50, 0.42, 0.31, 1.00);
    pub const DARKBROWN: Color = Color::new(0.30, 0.25, 0.18, 1.00);
    pub const WHITE: Color = Color::new(1.00, 1.00, 1.00, 1.00);
    pub const BLACK: Color = Color::new(0.00, 0.00, 0.00, 1.00);
    pub const BLANK: Color = Color::new(0.00, 0.00, 0.00, 0.00);
    pub const MAGENTA: Color = Color::new(1.00, 0.00, 1.00, 1.00);
}

#[rustfmt::skip]
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let r;
    let g;
    let b;

    if s == 0.0 {  r = l; g = l; b = l; }
    else {
        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 { t += 1.0 }
            if t > 1.0 { t -= 1.0 }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        g = hue_to_rgb(p, q, h);
        b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    }

    Color::new(r, g, b, 1.0)
}

pub fn rgb_to_hsl(color: Color) -> (f32, f32, f32) {
    fn max(a: f32, b: f32) -> f32 {
        if a > b {
            a
        } else {
            b
        }
    }
    fn min(a: f32, b: f32) -> f32 {
        if a < b {
            a
        } else {
            b
        }
    }

    let mut h: f32;
    let s: f32;
    let l: f32;

    let Color { r, g, b, .. } = color;

    let max = max(max(r, g), b);
    let min = min(min(r, g), b);

    // Luminosity is the average of the max and min rgb color intensities.
    l = (max + min) / 2.0;

    // Saturation
    let delta: f32 = max - min;
    if delta == 0.0 {
        // it's gray
        return (0.0, 0.0, l);
    }

    // it's not gray
    if l < 0.5 {
        s = delta / (max + min);
    } else {
        s = delta / (2.0 - max - min);
    }

    // Hue
    let r2 = (((max - r) / 6.0) + (delta / 2.0)) / delta;
    let g2 = (((max - g) / 6.0) + (delta / 2.0)) / delta;
    let b2 = (((max - b) / 6.0) + (delta / 2.0)) / delta;

    h = match max {
        x if x == r => b2 - g2,
        x if x == g => (1.0 / 3.0) + r2 - b2,
        _ => (2.0 / 3.0) + g2 - r2,
    };

    // Fix wraparounds
    if h < 0 as f32 {
        h += 1.0;
    } else if h > 1 as f32 {
        h -= 1.0;
    }

    (h, s, l)
}
