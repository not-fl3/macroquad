//! 3D shapes and models, loading 3d models from files, drawing 3D primitives.

use crate::{get_context, types::Color};

use glam::{vec2, vec3, Vec2, Vec3};
use quad_gl::{DrawMode, Texture2D};

fn draw_quad(vertices: [(Vec3, Vec2, Color); 4]) {
    let context = &mut get_context().draw_context;
    let indices = [0, 1, 2, 0, 2, 3];
    let quad = [
        (
            [vertices[0].0.x(), vertices[0].0.y(), vertices[0].0.z()],
            [vertices[0].1.x(), vertices[0].1.y()],
            vertices[0].2.into(),
        ),
        (
            [vertices[1].0.x(), vertices[1].0.y(), vertices[1].0.z()],
            [vertices[1].1.x(), vertices[1].1.y()],
            vertices[1].2.into(),
        ),
        (
            [vertices[2].0.x(), vertices[2].0.y(), vertices[2].0.z()],
            [vertices[2].1.x(), vertices[2].1.y()],
            vertices[2].2.into(),
        ),
        (
            [vertices[3].0.x(), vertices[3].0.y(), vertices[3].0.z()],
            [vertices[3].1.x(), vertices[3].1.y()],
            vertices[3].2.into(),
        ),
    ];

    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&quad[..], &indices);
}

pub fn draw_line_3d(start: Vec3, end: Vec3, color: Color) {
    let context = &mut get_context().draw_context;
    let uv = [0., 0.];
    let color: [f32; 4] = color.into();
    let indices = [0, 1];

    let line = [
        ([start.x(), start.y(), start.z()], uv, color),
        ([end.x(), end.y(), end.z()], uv, color),
    ];
    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Lines);
    context.gl.geometry(&line[..], &indices);
}

/// Draw a grid centered at (0, 0, 0)
pub fn draw_grid(slices: u32, spacing: f32) {
    let half_slices = (slices as i32) / 2;
    for i in -half_slices..half_slices + 1 {
        let color = if i == 0 {
            Color::new(0.55, 0.55, 0.55, 0.75)
        } else {
            Color::new(0.75, 0.75, 0.75, 0.75)
        };

        draw_line_3d(
            vec3(i as f32 * spacing, 0., -half_slices as f32 * spacing),
            vec3(i as f32 * spacing, 0., half_slices as f32 * spacing),
            color,
        );
        draw_line_3d(
            vec3(-half_slices as f32 * spacing, 0., i as f32 * spacing),
            vec3(half_slices as f32 * spacing, 0., i as f32 * spacing),
            color,
        );
    }
}

pub fn draw_plane(center: Vec3, size: Vec2, texture: impl Into<Option<Texture2D>>, color: Color) {
    let v1 = (
        (center + vec3(-size.x(), 0., -size.y())).into(),
        vec2(0., 0.),
        color,
    );
    let v2 = (
        (center + vec3(-size.x(), 0., size.y())).into(),
        vec2(0., 1.),
        color,
    );
    let v3 = (
        (center + vec3(size.x(), 0., size.y())).into(),
        vec2(1., 1.),
        color,
    );
    let v4 = (
        (center + vec3(size.x(), 0., -size.y())).into(),
        vec2(1., 0.),
        color,
    );

    {
        let context = &mut get_context().draw_context;
        context.gl.texture(texture.into());
    }
    draw_quad([v1, v2, v3, v4]);
}

pub fn draw_cube(position: Vec3, size: Vec3, texture: impl Into<Option<Texture2D>>, color: Color) {
    let context = &mut get_context().draw_context;
    context.gl.texture(texture.into());

    let (x, y, z) = (position.x(), position.y(), position.z());
    let (width, height, length) = (size.x(), size.y(), size.z());

    // Front face
    let bl_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let bl_uv = vec2(0., 0.);
    let br_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let br_uv = vec2(1., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 1.);

    let tl_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let tl_uv = vec2(0., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);

    // Back face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 0.);
    let br_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let br_uv = vec2(1., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let tr_uv = vec2(1., 1.);

    let tl_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let tl_uv = vec2(0., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);

    // Top face
    let bl_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let tl_uv = vec2(1., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);

    // Bottom face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let tl_uv = vec2(1., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);

    // Right face
    let bl_pos = vec3(x + width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x + width / 2., y + height / 2., z - length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x + width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x + width / 2., y - height / 2., z + length / 2.);
    let tl_uv = vec2(1., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);

    // Left face
    let bl_pos = vec3(x - width / 2., y - height / 2., z - length / 2.);
    let bl_uv = vec2(0., 1.);
    let br_pos = vec3(x - width / 2., y + height / 2., z - length / 2.);
    let br_uv = vec2(0., 0.);

    let tr_pos = vec3(x - width / 2., y + height / 2., z + length / 2.);
    let tr_uv = vec2(1., 0.);

    let tl_pos = vec3(x - width / 2., y - height / 2., z + length / 2.);
    let tl_uv = vec2(1., 1.);

    draw_quad([
        (bl_pos, bl_uv, color),
        (br_pos, br_uv, color),
        (tr_pos, tr_uv, color),
        (tl_pos, tl_uv, color),
    ]);
}

pub fn draw_cube_wires(position: Vec3, size: Vec3, color: Color) {
    let (x, y, z) = (position.x(), position.y(), position.z());
    let (width, height, length) = (size.x(), size.y(), size.z());

    // Front Face

    // Bottom Line
    draw_line_3d(
        vec3(x - width / 2., y - height / 2., z + length / 2.),
        vec3(x + width / 2., y - height / 2., z + length / 2.),
        color,
    );

    // Left Line
    draw_line_3d(
        vec3(x + width / 2., y - height / 2., z + length / 2.),
        vec3(x + width / 2., y + height / 2., z + length / 2.),
        color,
    );

    // Top Line
    draw_line_3d(
        vec3(x + width / 2., y + height / 2., z + length / 2.),
        vec3(x - width / 2., y + height / 2., z + length / 2.),
        color,
    );

    // Right Line
    draw_line_3d(
        vec3(x - width / 2., y + height / 2., z + length / 2.),
        vec3(x - width / 2., y - height / 2., z + length / 2.),
        color,
    );

    // Back Face
    // Bottom Line
    draw_line_3d(
        vec3(x - width / 2., y - height / 2., z - length / 2.),
        vec3(x + width / 2., y - height / 2., z - length / 2.),
        color,
    );

    // Left Line
    draw_line_3d(
        vec3(x + width / 2., y - height / 2., z - length / 2.),
        vec3(x + width / 2., y + height / 2., z - length / 2.),
        color,
    );

    // Top Line
    draw_line_3d(
        vec3(x + width / 2., y + height / 2., z - length / 2.),
        vec3(x - width / 2., y + height / 2., z - length / 2.),
        color,
    );

    // Right Line
    draw_line_3d(
        vec3(x - width / 2., y + height / 2., z - length / 2.),
        vec3(x - width / 2., y - height / 2., z - length / 2.),
        color,
    );

    // Top Face
    // Left Line
    draw_line_3d(
        vec3(x - width / 2., y + height / 2., z + length / 2.),
        vec3(x - width / 2., y + height / 2., z - length / 2.),
        color,
    );

    // Right Line
    draw_line_3d(
        vec3(x + width / 2., y + height / 2., z + length / 2.),
        vec3(x + width / 2., y + height / 2., z - length / 2.),
        color,
    );

    // Bottom Face
    // Left Line
    draw_line_3d(
        vec3(x - width / 2., y - height / 2., z + length / 2.),
        vec3(x - width / 2., y - height / 2., z - length / 2.),
        color,
    );

    // Right Line
    draw_line_3d(
        vec3(x + width / 2., y - height / 2., z + length / 2.),
        vec3(x + width / 2., y - height / 2., z - length / 2.),
        color,
    );
}

pub fn draw_sphere(center: Vec3, radius: f32, texture: impl Into<Option<Texture2D>>, color: Color) {
    let context = &mut get_context().draw_context;

    let rings: usize = 16;
    let slices: usize = 16;

    let color: [f32; 4] = color.into();
    let scale = vec3(radius, radius, radius);

    context.gl.texture(texture.into());
    context.gl.draw_mode(DrawMode::Triangles);

    for i in 0..rings + 1 {
        for j in 0..slices {
            use std::f32::consts::PI;

            let pi34 = PI / 2. * 3.;
            let pi2 = PI * 2.;
            let i = i as f32;
            let j = j as f32;
            let rings: f32 = rings as _;
            let slices: f32 = slices as _;

            let v1 = vec3(
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * i).sin(),
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).cos(),
            );
            let uv1 = [i / rings, j / slices];
            let v2 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv2 = [(i + 1.) / rings, (j + 1.) / slices];
            let v3 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * (j * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * (j * pi2 / slices).cos(),
            );
            let uv3 = [(i + 1.) / rings, j / slices];

            context.gl.geometry(
                &[
                    ((v1 * scale + center).into(), uv1, color),
                    ((v2 * scale + center).into(), uv2, color),
                    ((v3 * scale + center).into(), uv3, color),
                ],
                &[0, 1, 2],
            );

            let v1 = vec3(
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * i).sin(),
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).cos(),
            );
            let uv1 = [i / rings, j / slices];
            let v2 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv2 = [i / rings, (j + 1.) / slices];
            let v3 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv3 = [(i + 1.) / rings, (j + 1.) / slices];

            context.gl.geometry(
                &[
                    ((v1 * scale + center).into(), uv1, color),
                    ((v2 * scale + center).into(), uv2, color),
                    ((v3 * scale + center).into(), uv3, color),
                ],
                &[0, 1, 2],
            );
        }
    }
}
