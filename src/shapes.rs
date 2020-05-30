//! 2D shapes rendering

use crate::{
    get_context,
    types::{Color, Rect},
};

use glam::vec2;
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
            let top_coord = atlas.font_size as f32 - font_data.height_over_line;

            total_width += font_data.size.0 + font_data.right_padding;

            let dest = Rect::new(
                left_coord + x,
                top_coord + y - 5.,
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

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let context = &mut get_context().draw_context;

    context.draw_rectangle(x, y, w, h, color);
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
    border_color: Color,
    fill_color: Color,
) {
    let context = &mut get_context().draw_context;

    const NUM_DIVISIONS: u32 = 6;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertices.push(Vertex::new(x, y, 0., 0., 0., fill_color));
    let mut n = 0;
    for i in 0..NUM_DIVISIONS {
        let d = std::f32::consts::PI / 2.;
        // internal vertices
        {
            let r = size - border;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, fill_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, fill_color);
            vertices.push(vertex);

            indices.extend_from_slice(&[0, n + 1, n + 2]);
        }

        // duplicate internal vertices with border_color
        {
            let r = size - border;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);
        }

        // external border
        {
            let r = size;
            let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);

            let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).cos();
            let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2. - d).sin();
            let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, border_color);
            vertices.push(vertex);
        }

        indices.extend_from_slice(&[n + 5, n + 3, n + 4]);
        indices.extend_from_slice(&[n + 5, n + 4, n + 6]);

        n += 6;
    }

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    let context = &mut get_context().draw_context;

    const NUM_DIVISIONS: u32 = 200;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertices.push(Vertex::new(x, y, 0., 0., 0., color));
    for i in 0..NUM_DIVISIONS + 1 {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let vertex = Vertex::new(x + r * rx, y + r * ry, 0., rx, ry, color);

        vertices.push(vertex);

        if i != NUM_DIVISIONS {
            indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
        }
    }

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    const NUM_DIVISIONS: u32 = 200;

    for i in 0..NUM_DIVISIONS {
        let rx = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = (i as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let p0 = vec2(x + r * rx, y + r * ry);

        let rx = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).cos();
        let ry = ((i + 1) as f32 / NUM_DIVISIONS as f32 * std::f32::consts::PI * 2.).sin();

        let p1 = vec2(x + r * rx, y + r * ry);

        draw_line(p0.x(), p0.y(), p1.x(), p1.y(), thickness, color);
    }
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    let context = &mut get_context().draw_context;
    context.draw_line(x1, y1, x2, y2, thickness, color);
}
