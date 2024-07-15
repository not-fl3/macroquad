//! 2D shapes rendering.

use crate::{color::Color, get_context};

use crate::quad_gl::{DrawMode, Vertex};
use glam::{vec2, vec3, vec4, Mat4, Vec2};

/// Draws a solid triangle between points `v1`, `v2`, and `v3` with a given `color`.
pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
    let context = get_context();

    let vertices = [
        Vertex::new(v1.x, v1.y, 0., 0., 0., color),
        Vertex::new(v2.x, v2.y, 0., 0., 0., color),
        Vertex::new(v3.x, v3.y, 0., 0., 0., color),
    ];

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
///
/// # Deprecation
/// Due to a bug, this function does not actually draw lines that are `thickness` thick, but only `thickness / 2.`.
/// To preserve backwards compability, this function was not changed, and
/// a new function `draw_rectangle_lines_fixed` was added, which does not contain the bug.
///
/// See https://github.com/not-fl3/macroquad/issues/704 for more details.
#[deprecated(
    since = "0.4.12",
    note = "incorrect thickness handling, see issue #704. use `draw_rectangle_lines_fixed`"
)]
pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    draw_rectangle_lines_fixed(x, y, w, h, thickness / 2., color);
}

/// Draws a rectangle outline with its top-left corner at `[x, y]` with size `[w, h]` (width going to
/// the right, height going down), with a given line `thickness` and `color`.
pub fn draw_rectangle_lines_fixed(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let context = get_context();
    let t = thickness;

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

pub fn draw_rectangle_lines_ex(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    thickness: f32,
    params: DrawRectangleParams,
) {
    let context = get_context();
    let tx = thickness / w;
    let ty = thickness / h;

    let transform_matrix = Mat4::from_translation(vec3(x, y, 0.0))
        * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), params.rotation)
        * Mat4::from_scale(vec3(w, h, 1.0));

    #[rustfmt::skip]
    let v = [
        transform_matrix * vec4( 0.0 - params.offset.x,  0.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 0.0 - params.offset.x,  1.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x,  1.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x,  0.0 - params.offset.y, 0.0, 1.0),

        transform_matrix * vec4( 0.0 - params.offset.x + tx,  0.0 - params.offset.y + ty, 0.0, 1.0),
        transform_matrix * vec4( 0.0 - params.offset.x + tx,  1.0 - params.offset.y - ty, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x - tx,  1.0 - params.offset.y - ty, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x - tx,  0.0 - params.offset.y + ty, 0.0, 1.0),
    ];

    // TODO: fix UVs
    #[rustfmt::skip]
    let vertices = [
        Vertex::new(v[0].x, v[0].y, v[0].z, 0.0, 1.0, params.color),
        Vertex::new(v[1].x, v[1].y, v[1].z, 1.0, 0.0, params.color),
        Vertex::new(v[2].x, v[2].y, v[2].z, 1.0, 1.0, params.color),
        Vertex::new(v[3].x, v[3].y, v[3].z, 1.0, 0.0, params.color),

        Vertex::new(v[4].x, v[4].y, v[4].z, 0.0, 0.0, params.color),
        Vertex::new(v[5].x, v[5].y, v[5].z, 0.0, 0.0, params.color),
        Vertex::new(v[6].x, v[6].y, v[6].z, 0.0, 0.0, params.color),
        Vertex::new(v[7].x, v[7].y, v[7].z, 0.0, 0.0, params.color),
    ];
    #[rustfmt::skip]
    let indices: [u16; 24] = [
        0, 4, 3,
        4, 3, 7,
        4, 0, 1,
        4, 5, 1,
        1, 5, 6,
        1, 6, 2,
        2, 3, 6,
        3, 6, 7,
    ];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

#[derive(Debug, Clone)]
pub struct DrawRectangleParams {
    /// Adds an offset to the position
    /// E.g. offset (0,0) positions the rectangle at the top left corner of the screen, while offset
    /// (0.5, 0.5) centers it
    pub offset: Vec2,

    /// Rotation in radians
    pub rotation: f32,

    pub color: Color,
}

impl Default for DrawRectangleParams {
    fn default() -> Self {
        Self {
            offset: vec2(0.0, 0.0),
            rotation: 0.0,
            color: Color::from_rgba(255, 255, 255, 255),
        }
    }
}

/// Draws a solid rectangle with its position at `[x, y]` with size `[w, h]`,
/// with parameters.
pub fn draw_rectangle_ex(x: f32, y: f32, w: f32, h: f32, params: DrawRectangleParams) {
    let context = get_context();
    let transform_matrix = Mat4::from_translation(vec3(x, y, 0.0))
        * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), params.rotation)
        * Mat4::from_scale(vec3(w, h, 1.0));

    #[rustfmt::skip]
    let v = [
        transform_matrix * vec4( 0.0 - params.offset.x,  0.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 0.0 - params.offset.x,  1.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x,  1.0 - params.offset.y, 0.0, 1.0),
        transform_matrix * vec4( 1.0 - params.offset.x,  0.0 - params.offset.y, 0.0, 1.0),
    ];

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(v[0].x, v[0].y, v[0].z, 0.0, 0.0, params.color),
        Vertex::new(v[1].x, v[1].y, v[1].z, 1.0, 0.0, params.color),
        Vertex::new(v[2].x, v[2].y, v[2].z, 1.0, 1.0, params.color),
        Vertex::new(v[3].x, v[3].y, v[3].z, 0.0, 1.0, params.color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
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
    for i in 0..=sides {
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
    draw_arc(x, y, sides, radius, rotation, thickness, 360.0, color);
}

/// Draws a solid circle centered at `[x, y]` with a given radius `r` and `color`.
pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 20, r, 0., color);
}

/// Draws a circle outline centered at `[x, y]` with a given radius, line `thickness` and `color`.
pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    draw_poly_lines(x, y, 30, r, 0., thickness, color);
}

/// Draws a solid ellipse centered at `[x, y]` with a given size `[w, h]`,
/// clockwise `rotation` (in degrees) and `color`.
pub fn draw_ellipse(x: f32, y: f32, w: f32, h: f32, rotation: f32, color: Color) {
    let sides = 20;
    let context = get_context();

    let mut vertices = Vec::<Vertex>::with_capacity(sides as usize + 2);
    let mut indices = Vec::<u16>::with_capacity(sides as usize * 3);

    let rot = rotation.to_radians();
    let sr = rot.sin();
    let cr = rot.cos();
    vertices.push(Vertex::new(x, y, 0., 0., 0., color));
    for i in 0..=sides {
        let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2.).sin();

        let px = w * rx;
        let py = h * ry;
        let rotated_x = px * cr - py * sr;
        let rotated_y = py * cr + px * sr;
        let vertex = Vertex::new(x + rotated_x, y + rotated_y, 0., rx, ry, color);

        vertices.push(vertex);

        if i != sides {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

/// Draws an ellipse outline centered at `[x, y]` with a given size `[w, h]`,
/// clockwise `rotation` (in degrees), line `thickness` and `color`.
pub fn draw_ellipse_lines(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    thickness: f32,
    color: Color,
) {
    let sides = 20;

    let rot = rotation.to_radians();
    let sr = rot.sin();
    let cr = rot.cos();
    for i in 0..sides {
        let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2.).sin();
        let px = w * rx;
        let py = h * ry;
        let rotated_x = px * cr - py * sr;
        let rotated_y = py * cr + px * sr;

        let p0 = vec2(x + rotated_x, y + rotated_y);

        let rx = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2.).cos();
        let ry = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2.).sin();
        let px = w * rx;
        let py = h * ry;
        let rotated_x = px * cr - py * sr;
        let rotated_y = py * cr + px * sr;

        let p1 = vec2(x + rotated_x, y + rotated_y);

        draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
    }
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
    if tlen < f32::EPSILON {
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

/// Draw arc from `rotation`(in degrees) to `arc + rotation` (`arc` in degrees),
/// centered at `[x, y]` with a given number of `sides`, `radius`, line `thickness`, and `color`.
pub fn draw_arc(
    x: f32,
    y: f32,
    sides: u8,
    radius: f32,
    rotation: f32,
    thickness: f32,
    arc: f32,
    color: Color,
) {
    let rot = rotation.to_radians();
    let part = arc.to_radians();

    for i in 0..sides {
        let angle = i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot;
        let p0 = vec2(x + radius * angle.cos(), y + radius * angle.sin());

        let angle = (i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2. + rot;
        if angle > part + rot {
            continue;
        }
        let p1 = vec2(x + radius * angle.cos(), y + radius * angle.sin());

        let mid = p0.midpoint(p1);
        let v = (vec2(x, y) - mid).normalize() * (thickness / 2.);
        let p0 = p0 + v;
        let p1 = p1 + v;
        draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
    }
}
