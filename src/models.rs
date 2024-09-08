//! 3D shapes and models, loading 3d models from files, drawing 3D primitives.

use crate::{color::Color, get_context};

use crate::{quad_gl::DrawMode, texture::Texture2D};
use glam::{vec2, vec3, vec4, Quat, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [u8; 4],
    /// Normal is not used by macroquad and is completely optional.
    /// Might be usefull for custom shaders.
    /// While normal is not used by macroquad, it is completely safe to use it
    /// to pass arbitary user data, hence Vec4.
    pub normal: Vec4,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: Color) -> Vertex {
        Vertex {
            position: vec3(x, y, z),
            uv: vec2(u, v),
            color: color.into(),
            normal: vec4(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn new2(position: Vec3, uv: Vec2, color: Color) -> Vertex {
        Vertex {
            position,
            uv,
            color: color.into(),
            normal: vec4(0.0, 0.0, 0.0, 0.0),
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: Option<Texture2D>,
}

pub fn draw_mesh(mesh: &Mesh) {
    let context = get_context();

    context.gl.texture(mesh.texture.as_ref());
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&mesh.vertices[..], &mesh.indices[..]);
}

fn draw_quad(vertices: [Vertex; 4]) {
    let context = get_context();
    let indices = [0, 1, 2, 0, 2, 3];
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

pub fn draw_line_3d(start: Vec3, end: Vec3, color: Color) {
    let context = get_context();
    let uv = vec2(0., 0.);
    let indices = [0, 1];

    let line = [Vertex::new2(start, uv, color), Vertex::new2(end, uv, color)];

    context.gl.texture(None);
    context.gl.draw_mode(DrawMode::Lines);
    context.gl.geometry(&line, &indices);
}

/// Draw a grid centered at (0, 0, 0)
pub fn draw_grid(slices: u32, spacing: f32, axes_color: Color, other_color: Color) {
    draw_grid_ex(
        slices,
        spacing,
        axes_color,
        other_color,
        vec3(0., 0., 0.),
        Quat::IDENTITY,
    );
}

/// Draw a rotated grid centered at a specified point
pub fn draw_grid_ex(
    slices: u32,
    spacing: f32,
    axes_color: Color,
    other_color: Color,
    center: Vec3,
    rotation: Quat,
) {
    let half_slices = (slices as i32) / 2;
    for i in -half_slices..half_slices + 1 {
        let color = if i == 0 { axes_color } else { other_color };

        let start = vec3(i as f32 * spacing, 0., -half_slices as f32 * spacing);
        let end = vec3(i as f32 * spacing, 0., half_slices as f32 * spacing);

        draw_line_3d(
            rotation.mul_vec3(start) + center,
            rotation.mul_vec3(end) + center,
            color,
        );

        let start = vec3(-half_slices as f32 * spacing, 0., i as f32 * spacing);
        let end = vec3(half_slices as f32 * spacing, 0., i as f32 * spacing);

        draw_line_3d(
            rotation.mul_vec3(start) + center,
            rotation.mul_vec3(end) + center,
            color,
        );
    }
}

pub fn draw_plane(center: Vec3, size: Vec2, texture: Option<&Texture2D>, color: Color) {
    let v1 = Vertex::new2(center + vec3(-size.x, 0., -size.y), vec2(0., 0.), color);
    let v2 = Vertex::new2(center + vec3(-size.x, 0., size.y), vec2(0., 1.), color);
    let v3 = Vertex::new2(center + vec3(size.x, 0., size.y), vec2(1., 1.), color);
    let v4 = Vertex::new2(center + vec3(size.x, 0., -size.y), vec2(1., 0.), color);

    {
        let context = get_context();
        context.gl.texture(texture);
    }
    draw_quad([v1, v2, v3, v4]);
}

/// Draw an affine (2D) parallelogram at given position, as two triangles.
///
/// The drawn parallelogram will have the vertices: `offset`, `offset + e1`, `offset + e2` and `offset + e1 + e2`
///
/// # Arguments
///
/// * `offset` - Offset of the first point from the origin
/// * `e1`, `e2` - Base vectors for the parallelogram
/// * `texture` - Optional [Texture2D] to apply, which will be streched on the entire shape (todo!
/// support custom uv values per vertex)
/// * `color` - The [Color] to draw the parallelogram
///
/// # Examples
///
/// Draw an axis aligned rectangle
/// ```no_run
/// # use macroquad::prelude::*;
/// draw_affine_parallelogram(Vec3::ZERO, 3. * Vec3::X, 5. * Vec3::Z, None, RED);
/// ```
pub fn draw_affine_parallelogram(
    offset: Vec3,
    e1: Vec3,
    e2: Vec3,
    texture: Option<&Texture2D>,
    color: Color,
) {
    let v1 = Vertex::new2(offset, vec2(0., 0.), color);
    let v2 = Vertex::new2(offset + e1, vec2(0., 1.), color);
    let v3 = Vertex::new2(offset + e1 + e2, vec2(1., 1.), color);
    let v4 = Vertex::new2(offset + e2, vec2(1., 0.), color);

    {
        let context = get_context();
        context.gl.texture(texture);
    }
    draw_quad([v1, v2, v3, v4]);
}

/// Draw an affine (3D) parallelepiped at given position, using six parallelograms.
///
/// The drawn parallelepiped will be built from the followwing parallelograms:
///
/// * `offset, offset + e1, offset + e2`
/// * `offset, offset + e2, offset + e3`
/// * `offset, offset + e1, offset + e3`
/// * `offset, offset + e1 + e2, offset + e1 + e3`
/// * `offset, offset + e2 + e1, offset + e2 + e3`
/// * `offset, offset + e3 + e1, offset + e3 + e2`
///
/// # Arguments
///
/// * `offset` - Offset of the first point from the origin
/// * `e1`, `e2`, `e3` - Base vectors for the parallelepiped
/// * `texture` - Optional [Texture2D] to apply, which will repeat on each face (todo!
/// support custom uv values per vertex, multiple textures?)
/// * `color` - The [Color] to draw the parallelepiped (todo! support color per face?)
///
/// # Examples
///
/// Draw an axis aligned cube
/// ```no_run
/// # use macroquad::prelude::*;
/// draw_affine_parallelepiped(Vec3::ZERO, 3. * Vec3::X, 2. * Vec3::Y, 5. * Vec3::Z, None, RED);
/// ```
pub fn draw_affine_parallelepiped(
    offset: Vec3,
    e1: Vec3,
    e2: Vec3,
    e3: Vec3,
    texture: Option<&Texture2D>,
    color: Color,
) {
    draw_affine_parallelogram(offset, e1, e2, texture, color);
    draw_affine_parallelogram(offset, e1, e3, texture, color);
    draw_affine_parallelogram(offset, e2, e3, texture, color);

    draw_affine_parallelogram(offset + e1, e2, e3, texture, color);
    draw_affine_parallelogram(offset + e2, e1, e3, texture, color);
    draw_affine_parallelogram(offset + e3, e1, e2, texture, color);
}

pub fn draw_cube(position: Vec3, size: Vec3, texture: Option<&Texture2D>, color: Color) {
    let context = get_context();
    context.gl.texture(texture);

    let (x, y, z) = (position.x, position.y, position.z);
    let (width, height, length) = (size.x, size.y, size.z);

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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
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
        Vertex::new2(bl_pos, bl_uv, color),
        Vertex::new2(br_pos, br_uv, color),
        Vertex::new2(tr_pos, tr_uv, color),
        Vertex::new2(tl_pos, tl_uv, color),
    ]);
}

pub fn draw_cube_wires(position: Vec3, size: Vec3, color: Color) {
    let (x, y, z) = (position.x, position.y, position.z);
    let (width, height, length) = (size.x, size.y, size.z);

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

#[derive(Debug, Clone)]
pub struct DrawSphereParams {
    pub rings: usize,
    pub slices: usize,
    pub draw_mode: DrawMode,
}

impl Default for DrawSphereParams {
    fn default() -> DrawSphereParams {
        DrawSphereParams {
            rings: 16,
            slices: 16,
            draw_mode: DrawMode::Triangles,
        }
    }
}

pub fn draw_sphere(center: Vec3, radius: f32, texture: Option<&Texture2D>, color: Color) {
    draw_sphere_ex(center, radius, texture, color, Default::default());
}

pub fn draw_sphere_wires(center: Vec3, radius: f32, texture: Option<&Texture2D>, color: Color) {
    let params = DrawSphereParams {
        draw_mode: DrawMode::Lines,
        ..Default::default()
    };
    draw_sphere_ex(center, radius, texture, color, params);
}

pub fn draw_sphere_ex(
    center: Vec3,
    radius: f32,
    texture: Option<&Texture2D>,
    color: Color,
    params: DrawSphereParams,
) {
    let context = get_context();

    let rings = params.rings;
    let slices = params.slices;

    let scale = vec3(radius, radius, radius);

    context.gl.texture(texture);
    context.gl.draw_mode(params.draw_mode);

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
            let uv1 = vec2(i / rings, j / slices);
            let v2 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv2 = vec2((i + 1.) / rings, (j + 1.) / slices);
            let v3 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * (j * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * (j * pi2 / slices).cos(),
            );
            let uv3 = vec2((i + 1.) / rings, j / slices);

            context.gl.geometry(
                &[
                    Vertex::new2(v1 * scale + center, uv1, color),
                    Vertex::new2(v2 * scale + center, uv2, color),
                    Vertex::new2(v3 * scale + center, uv3, color),
                ],
                &[0, 1, 2],
            );

            let v1 = vec3(
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * i).sin(),
                (pi34 + (PI / (rings + 1.)) * i).cos() * (j * pi2 / slices).cos(),
            );
            let uv1 = vec2(i / rings, j / slices);
            let v2 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv2 = vec2(i / rings, (j + 1.) / slices);
            let v3 = vec3(
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).sin(),
                (pi34 + (PI / (rings + 1.)) * (i + 1.)).cos() * ((j + 1.) * pi2 / slices).cos(),
            );
            let uv3 = vec2((i + 1.) / rings, (j + 1.) / slices);

            context.gl.geometry(
                &[
                    Vertex::new2(v1 * scale + center, uv1, color),
                    Vertex::new2(v2 * scale + center, uv2, color),
                    Vertex::new2(v3 * scale + center, uv3, color),
                ],
                &[0, 1, 2],
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawCylinderParams {
    pub sides: usize,
    pub draw_mode: DrawMode,
}

impl Default for DrawCylinderParams {
    fn default() -> DrawCylinderParams {
        DrawCylinderParams {
            sides: 16,
            draw_mode: DrawMode::Triangles,
        }
    }
}

pub fn draw_cylinder(
    position: Vec3,
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    texture: Option<&Texture2D>,
    color: Color,
) {
    draw_cylinder_ex(
        position,
        radius_top,
        radius_bottom,
        height,
        texture,
        color,
        Default::default(),
    );
}

pub fn draw_cylinder_wires(
    position: Vec3,
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    texture: Option<&Texture2D>,
    color: Color,
) {
    let params = DrawCylinderParams {
        draw_mode: DrawMode::Lines,
        ..Default::default()
    };
    draw_cylinder_ex(
        position,
        radius_top,
        radius_bottom,
        height,
        texture,
        color,
        params,
    );
}

//Note: can also be used to draw a cone by setting radius_top or radius_bottom to 0
pub fn draw_cylinder_ex(
    position: Vec3,
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    texture: Option<&Texture2D>,
    color: Color,
    params: DrawCylinderParams,
) {
    let context = get_context();

    let sides = params.sides;

    context.gl.texture(texture);
    context.gl.draw_mode(params.draw_mode);

    use std::f32::consts::PI;
    let angle_step = PI * 2.0 / sides as f32;
    //draw body
    for i in 0..sides + 1 {
        let i = i as f32;
        //bottom left
        let v1 = vec3(
            (i * angle_step).sin() * radius_bottom,
            0.0,
            (i * angle_step).cos() * radius_bottom,
        );
        //bottom right
        let v2 = vec3(
            ((i + 1.0) * angle_step).sin() * radius_bottom,
            0.0,
            ((i + 1.0) * angle_step).cos() * radius_bottom,
        );
        //top right
        let v3 = vec3(
            ((i + 1.0) * angle_step).sin() * radius_top,
            height,
            ((i + 1.0) * angle_step).cos() * radius_top,
        );

        context.gl.geometry(
            &[
                Vertex::new2(v1 + position, vec2(0.0, 0.0), color),
                Vertex::new2(v2 + position, vec2(1.0, 0.0), color),
                Vertex::new2(v3 + position, vec2(1.0, 1.0), color),
            ],
            &[0, 1, 2],
        );

        //top left
        let v1 = vec3(
            (i * angle_step).sin() * radius_top,
            height,
            (i * angle_step).cos() * radius_top,
        );
        //bottom left
        let v2 = vec3(
            (i * angle_step).sin() * radius_bottom,
            0.0,
            (i * angle_step).cos() * radius_bottom,
        );
        //top right
        let v3 = vec3(
            ((i + 1.0) * angle_step).sin() * radius_top,
            height,
            ((i + 1.0) * angle_step).cos() * radius_top,
        );

        context.gl.geometry(
            &[
                Vertex::new2(v1 + position, vec2(0.0, 0.0), color),
                Vertex::new2(v2 + position, vec2(1.0, 0.0), color),
                Vertex::new2(v3 + position, vec2(1.0, 1.0), color),
            ],
            &[0, 1, 2],
        );
    }

    //draw cap
    for i in 0..sides + 1 {
        let i = i as f32;
        let v1 = vec3(0.0, height, 0.0);
        let v2 = vec3(
            (i * angle_step).sin() * radius_top,
            height,
            (i * angle_step).cos() * radius_top,
        );
        let v3 = vec3(
            ((i + 1.0) * angle_step).sin() * radius_top,
            height,
            ((i + 1.0) * angle_step).cos() * radius_top,
        );

        context.gl.geometry(
            &[
                Vertex::new2(v1 + position, vec2(0.0, 0.0), color),
                Vertex::new2(v2 + position, vec2(1.0, 0.0), color),
                Vertex::new2(v3 + position, vec2(1.0, 1.0), color),
            ],
            &[0, 1, 2],
        );
    }

    //draw base
    for i in 0..sides + 1 {
        let i = i as f32;
        let v1 = vec3(0.0, 0.0, 0.0);
        let v2 = vec3(
            (i * angle_step).sin() * radius_bottom,
            0.0,
            (i * angle_step).cos() * radius_bottom,
        );
        let v3 = vec3(
            ((i + 1.0) * angle_step).sin() * radius_bottom,
            0.0,
            ((i + 1.0) * angle_step).cos() * radius_bottom,
        );

        context.gl.geometry(
            &[
                Vertex::new2(v1 + position, vec2(0.0, 0.0), color),
                Vertex::new2(v2 + position, vec2(1.0, 0.0), color),
                Vertex::new2(v3 + position, vec2(1.0, 1.0), color),
            ],
            &[0, 1, 2],
        );
    }
}
