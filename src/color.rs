//! Color types and helpers.

pub use quad_gl::{colors::*, Color};

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
            return p;
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
