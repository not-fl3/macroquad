//! 2D shapes rendering.

use crate::{color, color::mix_colors, color::Color, get_context, math::Rect};

use crate::quad_gl::{DrawMode, Vertex};
use glam::{vec2, Vec2};

/// Draws a solid triangle between points `v1`, `v2`, and `v3` with a given `color`.
pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
    let context = get_context();

    let mut vertices = Vec::<Vertex>::with_capacity(3);

    vertices.push(Vertex::new(v1.x, v1.y, 0., 0., 0., color));
    vertices.push(Vertex::new(v2.x, v2.y, 0., 0., 0., color));
    vertices.push(Vertex::new(v3.x, v3.y, 0., 0., 0., color));
    let indices: [u16; 3] = [0, 1, 2];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

/// Draws a triangle outline between points `v1`, `v2`, and `v3` with a given line `thickness` and `color`.
pub fn draw_triangle_lines(v1: Vec2, v2: Vec2, v3: Vec2, thickness: f32, color: Color) {
    draw_line(v1.x, v1.y, v2.x, v2.y, thickness, color);
    draw_line(v2.x, v2.y, v3.x, v3.y, thickness, color);
    draw_line(v3.x, v3.y, v1.x, v1.y, thickness, color);
}

/// Draws a solid rectangle with its top-left corner at `[x, y]` with size `[w, h]` (width going to
/// the right, height going down), with a given `color`.
pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = get_context();

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , 0., 0.0, 0.0, color),
        Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
        Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
        Vertex::new(x    , y + h, 0., 0.0, 1.0, color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

/// Draws a rectangle outline with its top-left corner at `[x, y]` with size `[w, h]` (width going to
/// the right, height going down), with a given line `thickness` and `color`.
pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let context = get_context();
    let t = thickness / 2.;

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , 0., 0.0, 1.0, color),
        Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
        Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
        Vertex::new(x    , y + h, 0., 0.0, 0.0, color),
        //inner rectangle
        Vertex::new(x + t    , y + t    , 0., 0.0, 0.0, color),
        Vertex::new(x + w - t, y + t    , 0., 0.0, 0.0, color),
        Vertex::new(x + w - t, y + h - t, 0., 0.0, 0.0, color),
        Vertex::new(x + t    , y + h - t, 0., 0.0, 0.0, color),
    ];
    let indices: [u16; 24] = [
        0, 1, 4, 1, 4, 5, 1, 5, 6, 1, 2, 6, 3, 7, 2, 2, 7, 6, 0, 4, 3, 3, 4, 7,
    ];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

#[derive(Debug, Clone)]
pub struct DrawRectangleParams {
    /// Rotation in radians
    pub rotation: f32,
    /// Rotate around this point.
    /// When `None`, rotate around the rectangle's center.
    /// When `Some`, the coordinates are in world-space.
    /// E.g. pivot (0,0) rotates around the top left corner of the world, not of the
    /// rectangle.
    pub pivot: Option<Vec2>,
    /// Rectangle will be filled with gradient.
    /// Corner colors are specified in order: `[top_left, top_right, bottom_right, bottom_left]`
    /// Overriders `color`.
    pub gradient: Option<[Color; 4]>,
    /// Color of the rectangle. Used if `gradient` is `None`.
    pub color: color::Color,
    /// If greater than 0.0, draws a rectangle outline with given `line_thickness`
    pub line_thickness: f32,
    /// Horizontal and vertical skew proportions
    pub skew: Vec2,
    /// Radius of rectangle's corners
    pub border_radius: f32,
    /// Number of segments used for drawing each corner
    /// Ignored if `border_radius` is 0.0
    pub border_radius_segments: u8,
}

impl Default for DrawRectangleParams {
    fn default() -> DrawRectangleParams {
        DrawRectangleParams {
            gradient: None,
            rotation: 0.,
            color: color::WHITE,
            line_thickness: 0.,
            pivot: None,
            skew: Vec2::ZERO,
            border_radius: 0.0,
            border_radius_segments: 5,
        }
    }
}

/// Note: last `Vertex` in returned `Vec` is center
fn rounded_rect(
    quart_vertices: u8,
    rect: crate::prelude::Rect,
    border_radius: f32,
    gradient: Option<&[Color; 4]>,
    center_color: Color,
    generate_indices: bool,
) -> (Vec<Vertex>, Vec<u16>) {
    use std::f32::consts::PI;
    let Rect { x, y, w, h } = rect;
    let mut indices: Vec<u16> = vec![];

    let rc = rect.center();
    let c0 = vec2(x + w - border_radius, y + border_radius);
    let c1 = vec2(x + border_radius, y + border_radius);
    let c2 = vec2(x + border_radius, y + h - border_radius);
    let c3 = vec2(x + w - border_radius, y + h - border_radius);

    let mut vertices: Vec<Vertex> = vec![];

    let v_num = quart_vertices * 4;

    vertices.extend((0..v_num).map(|i| {
        if generate_indices {
            if i < v_num - 1 {
                indices.extend([v_num as u16, (i) as u16, (i + 1) as u16]);
            } else {
                indices.extend([v_num as u16, (i) as u16, 0]);
            }
        }
        let (r, angle_cs) = match i {
            i if i >= quart_vertices * 3 => {
                // Top right quarter circle
                let angle = ((i - quart_vertices * 3) as f32 / (quart_vertices - 1) as f32) * PI
                    / 2.
                    + (3.) * PI / 2.;
                let angle_cs = vec2(angle.cos(), angle.sin());
                let r = c0 + (angle_cs * border_radius);
                (r, angle_cs)
            }
            i if i >= quart_vertices * 2 => {
                // Top left quarter circle
                let angle =
                    (i - quart_vertices * 2) as f32 / (quart_vertices - 1) as f32 * (PI / 2.) + PI;
                let angle_cs = vec2(angle.cos(), angle.sin());
                let r = c1 + (angle_cs * border_radius);
                (r, angle_cs)
            }
            i if i >= quart_vertices => {
                // Bottom right quarter circle
                let angle =
                    (i - quart_vertices) as f32 / (quart_vertices - 1) as f32 * PI / 2. + PI / 2.;
                let angle_cs = vec2(angle.cos(), angle.sin());
                let r = c2 + (angle_cs * border_radius);
                (r, angle_cs)
            }
            i => {
                // Bottom left quarter circle
                let angle = i as f32 / (quart_vertices - 1) as f32 * PI / 2.;
                let angle_cs = vec2(angle.cos(), angle.sin());
                let r = c3 + (angle_cs * border_radius);
                (r, angle_cs)
            }
        };

        let color = if let Some(gradient) = gradient {
            let h_rel = ((x + w) - r.x) / w;
            let v_rel = ((y + h) - r.y) / h;

            // Seems to work:
            // mix top left and top right colors based on horizontal distance
            let color_top = mix_colors(&gradient[0], &gradient[1], h_rel);
            // mix bot left and bot right colors based on horizontal distance
            let color_bot = mix_colors(&gradient[3], &gradient[2], h_rel);
            // mix results based on vertical distance
            mix_colors(&color_top, &color_bot, v_rel)
        } else {
            center_color
        };

        Vertex::new(r.x, r.y, 0., angle_cs.x, angle_cs.y, color)
    }));

    vertices.push(Vertex::new(rc.x, rc.y, 0., 0., 0., center_color));

    (vertices, indices)
}
fn skew_vertices(vertices: &mut [Vertex], skew: Vec2, pivot: Vec2) {
    vertices.iter_mut().for_each(|v| {
        let p = vec2(v.pos[0] - pivot.x, v.pos[1] - pivot.y);

        v.pos[0] = p.x + (skew.x * p.y) + pivot.x;
        v.pos[1] = p.y + (skew.y * p.x) + pivot.y;
    });
}
fn rotate_vertices(vertices: &mut [Vertex], rot: f32, pivot: Vec2) {
    let sin = rot.sin();
    let cos = rot.cos();
    vertices.iter_mut().for_each(|v| {
        let p = vec2(v.pos[0] - pivot.x, v.pos[1] - pivot.y);

        v.pos[0] = p.x * cos - p.y * sin + pivot.x;
        v.pos[1] = p.x * sin + p.y * cos + pivot.y;
    });
}

/// Draws a rectangle with its top-left corner at `[x, y]` with size `[w, h]` (width going to
/// the right, height going down), with a given `params`.
pub fn draw_rectangle_ex(x: f32, y: f32, w: f32, h: f32, param: &DrawRectangleParams) {
    let center = vec2(x + w / 2., y + h / 2.);
    let p = [
        vec2(x, y),
        vec2(x + w, y),
        vec2(x + w, y + h),
        vec2(x, y + h),
    ];

    let g = &param.gradient;
    let c = param.color;
    let t = param.line_thickness;

    let center_color = g.map_or(c, |g| {
        Color::new(
            g.iter().fold(0.0, |a, c| a + c.r) / 4.0,
            g.iter().fold(0.0, |a, c| a + c.g) / 4.0,
            g.iter().fold(0.0, |a, c| a + c.b) / 4.0,
            g.iter().fold(0.0, |a, c| a + c.a) / 4.0,
        )
    });

    let (mut outer_vertices, outer_indices): (Vec<Vertex>, Vec<u16>) = if param.border_radius > 0.0
    {
        // Rectangle with rounded corners
        rounded_rect(
            param.border_radius_segments * 2,
            Rect::new(x, y, w, h),
            param.border_radius,
            g.as_ref(),
            center_color,
            true,
        )
    } else {
        // Regular rectangle
        (
            vec![
                Vertex::new(p[0].x, p[0].y, 0., 0., 0., g.map_or(c, |g| g[0])),
                Vertex::new(p[1].x, p[1].y, 0., 1., 0., g.map_or(c, |g| g[1])),
                Vertex::new(p[2].x, p[2].y, 0., 1., 1., g.map_or(c, |g| g[2])),
                Vertex::new(p[3].x, p[3].y, 0., 0., 1., g.map_or(c, |g| g[3])),
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    };

    if param.skew != Vec2::ZERO {
        skew_vertices(&mut outer_vertices, param.skew, center);
    }

    let pivot = param.pivot.unwrap_or(center);

    if param.rotation != 0. {
        rotate_vertices(&mut outer_vertices, param.rotation, pivot);
    };

    let mut indices: Vec<u16>;
    if t > 0. {
        // Draw rectangle outline
        let mut inner_vertices: Vec<Vertex> = if param.border_radius > 0.0 {
            // Rectangle with rounded corners
            let mut inner_vert = rounded_rect(
                param.border_radius_segments * 2,
                Rect::new(x + t, y + t, w - 2. * t, h - 2. * t),
                param.border_radius * (w - 2. * t) / w,
                g.as_ref(),
                center_color,
                false,
            )
            .0;
            // We don't need center vertices when drawing outline
            outer_vertices.pop();
            inner_vert.pop();
            inner_vert
        } else {
            // Regular rectangle
            vec![
                Vertex::new(p[0].x + t, p[0].y + t, 0., 0., 0., g.map_or(c, |g| g[0])),
                Vertex::new(p[1].x - t, p[1].y + t, 0., 1., 0., g.map_or(c, |g| g[1])),
                Vertex::new(p[2].x - t, p[2].y - t, 0., 1., 1., g.map_or(c, |g| g[2])),
                Vertex::new(p[3].x + t, p[3].y - t, 0., 0., 1., g.map_or(c, |g| g[3])),
            ]
        };

        if param.skew != Vec2::ZERO {
            skew_vertices(&mut inner_vertices, param.skew, center);
        }
        if param.rotation != 0. {
            rotate_vertices(&mut inner_vertices, param.rotation, pivot);
        };

        let v_len = outer_vertices.len() as u16;

        // Merge outer and innver vertices
        outer_vertices.extend(&inner_vertices);

        // Generate indices
        indices = vec![];
        for i in 0..v_len {
            indices.extend([i, ((i + 1) % v_len as u16), v_len + (i as u16)]);
            indices.extend([
                i + v_len as u16,
                (i + 1) % v_len as u16,
                v_len + ((i + 1) % v_len) as u16,
            ]);
        }
    } else {
        indices = outer_indices;
    };

    let context = get_context();
    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&outer_vertices, &indices);
}

/// Draws an outlined solid hexagon centered at `[x, y]` with a radius `size`, outline thickness
/// defined by `border`, orientation defined by `vertical` (when `true`, the hexagon points along
/// the `y` axis), and colors for outline given by `border_color` and fill by `fill_color`.
pub fn draw_hexagon(
    x: f32,
    y: f32,
    size: f32,
    border: f32,
    vertical: bool,
    border_color: Color,
    fill_color: Color,
) {
    let rotation = if vertical { 90. } else { 0. };
    draw_poly(x, y, 6, size, rotation, fill_color);
    if border > 0. {
        draw_poly_lines(x, y, 6, size, rotation, border, border_color);
    }
}

/// Draws a solid regular polygon centered at `[x, y]` with a given number of `sides`, `radius`,
/// clockwise `rotation` (in degrees) and `color`.
pub fn draw_poly(x: f32, y: f32, sides: u8, radius: f32, rotation: f32, color: Color) {
    let context = get_context();

    let mut vertices = Vec::<Vertex>::with_capacity(sides as usize + 2);
    let mut indices = Vec::<u16>::with_capacity(sides as usize * 3);

    let rot = rotation.to_radians();
    vertices.push(Vertex::new(x, y, 0., 0., 0., color));
    for i in 0..sides + 1 {
        let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
        let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

        let vertex = Vertex::new(x + radius * rx, y + radius * ry, 0., rx, ry, color);

        vertices.push(vertex);

        if i != sides {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

/// Draws a regular polygon outline centered at `[x, y]` with a given number of `sides`, `radius`,
/// clockwise `rotation` (in degrees), line `thickness`, and `color`.
pub fn draw_poly_lines(
    x: f32,
    y: f32,
    sides: u8,
    radius: f32,
    rotation: f32,
    thickness: f32,
    color: Color,
) {
    let rot = rotation.to_radians();

    for i in 0..sides {
        let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
        let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

        let p0 = vec2(x + radius * rx, y + radius * ry);

        let rx = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
        let ry = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

        let p1 = vec2(x + radius * rx, y + radius * ry);

        draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
    }
}

/// Draws a solid circle centered at `[x, y]` with a given radius `r` and `color`.
pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 20, r, 0., color);
}

/// Draws a circle outline centered at `[x, y]` with a given radius, line `thickness` and `color`.
pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    draw_poly_lines(x, y, 20, r, 0., thickness, color);
}

/// Draws a line between points `[x1, y1]` and `[x2, y2]` with a given `thickness` and `color`.
pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = get_context();
    let dx = x2 - x1;
    let dy = y2 - y1;

    // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
    if tlen < std::f32::EPSILON {
        return;
    }
    let tx = nx / tlen;
    let ty = ny / tlen;

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(
        &[
            Vertex::new(x1 + tx, y1 + ty, 0., 0., 0., color),
            Vertex::new(x1 - tx, y1 - ty, 0., 0., 0., color),
            Vertex::new(x2 + tx, y2 + ty, 0., 0., 0., color),
            Vertex::new(x2 - tx, y2 - ty, 0., 0., 0., color),
        ],
        &[0, 1, 2, 2, 1, 3],
    );
}
