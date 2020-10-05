//! 2D shapes rendering.

use crate::{
    get_context,
    types::{Color, Rect},
};

use glam::{vec2, Vec2};
use quad_gl::{DrawMode, Vertex};

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let context = &mut get_context().draw_context;

    let atlas = context.ui.font_atlas.clone();

    let mut total_width = 0.;
    for character in text.chars() {
        if let Some(font_data) = atlas.character_infos.get(&character) {
            let font_data = font_data.scale(font_size);

            total_width += font_data.left_padding;

            let left_coord = total_width;
            let top_coord = font_size as f32 - font_data.height_over_line;

            total_width += font_data.size.0 + font_data.right_padding;

            let dest = Rect::new(
                left_coord + x,
                top_coord + y,
                font_data.size.0,
                font_data.size.1,
            );

            let source = Rect::new(
                font_data.tex_coords.0 * context.font_texture.width(),
                font_data.tex_coords.1 * context.font_texture.height(),
                font_data.tex_size.0 * context.font_texture.width(),
                font_data.tex_size.1 * context.font_texture.height(),
            );
            crate::texture::draw_texture_ex(
                context.font_texture,
                dest.x,
                dest.y,
                color,
                crate::texture::DrawTextureParams {
                    dest_size: Some(vec2(dest.w, dest.h)),
                    source: Some(source),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn measure_text(text: &str, font_size: f32) -> (f32, f32) {
    let context = &mut get_context().draw_context;

    let atlas = context.ui.font_atlas.clone();

    let mut width = 0.;
    let mut height: f32 = 0.;

    for character in text.chars() {
        if let Some(font_data) = atlas.character_infos.get(&character) {
            let font_data = font_data.scale(font_size);
            width += font_data.left_padding + font_data.size.0 + font_data.right_padding;
            height = height.max(font_data.size.1);
        }
    }
    return (width, height);
}

pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
    let context = &mut get_context().draw_context;

    let mut vertices = Vec::<Vertex>::with_capacity(3);

    vertices.push(Vertex::new(v1.x(), v1.y(), 0., 0., 0., color));
    vertices.push(Vertex::new(v2.x(), v2.y(), 0., 0., 0., color));
    vertices.push(Vertex::new(v3.x(), v3.y(), 0., 0., 0., color));
    let indices: [u16; 3] = [0, 1, 2];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_triangle_lines(v1: Vec2, v2: Vec2, v3: Vec2, thickness: f32, color: Color) {
    draw_line(v1.x(), v1.y(), v2.x(), v2.y(), thickness, color);
    draw_line(v2.x(), v2.y(), v3.x(), v3.y(), thickness, color);
    draw_line(v3.x(), v3.y(), v1.x(), v1.y(), thickness, color);
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    #[rustfmt::skip]
    let vertices = [
        Vertex::new(x    , y    , 0., 0.0, 1.0, color),
        Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
        Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
        Vertex::new(x    , y + h, 0., 0.0, 0.0, color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let t = thickness / 2.;

    draw_rectangle(x, y, w, t, color);
    draw_rectangle(x + w - t, y + t, t, h - t, color);
    draw_rectangle(x, y + h - t, w, t, color);
    draw_rectangle(x, y + t, t, h - t, color);
}

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

pub fn draw_poly(x: f32, y: f32, sides: u8, radius: f32, rotation: f32, color: Color) {
    let context = &mut get_context().draw_context;

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

        draw_line(p0.x(), p0.y(), p1.x(), p1.y(), thickness, color);
    }
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 200, r, 0., color);
}

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    draw_poly_lines(x, y, 200, r, 0., thickness, color);
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = &mut get_context().draw_context;
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
