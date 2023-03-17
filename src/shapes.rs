//! 2D shapes rendering.

use crate::color::Color;

use crate::{
    math::{Rect, Vec2},
    quad_gl::{DrawMode, Vertex},
    scene_graph::SpriteLayer,
};

impl SpriteLayer {
    // pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
    //     let context = get_context();

    //     let mut vertices = Vec::<Vertex>::with_capacity(3);

    //     vertices.push(Vertex::new(v1.x, v1.y, 0., 0., 0., color));
    //     vertices.push(Vertex::new(v2.x, v2.y, 0., 0., 0., color));
    //     vertices.push(Vertex::new(v3.x, v3.y, 0., 0., 0., color));
    //     let indices: [u16; 3] = [0, 1, 2];

    //     context.gl.texture(None);
    //     context.gl.draw_mode(DrawMode::Triangles);
    //     context.gl.geometry(&vertices, &indices);
    // }

    // pub fn draw_triangle_lines(v1: Vec2, v2: Vec2, v3: Vec2, thickness: f32, color: Color) {
    //     draw_line(v1.x, v1.y, v2.x, v2.y, thickness, color);
    //     draw_line(v2.x, v2.y, v3.x, v3.y, thickness, color);
    //     draw_line(v3.x, v3.y, v1.x, v1.y, thickness, color);
    // }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        #[rustfmt::skip]
        let vertices = [
            Vertex::new(x    , y    , 0., 0.0, 1.0, color),
            Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
            Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
            Vertex::new(x    , y + h, 0., 0.0, 0.0, color),
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        self.gl().texture(None);
        self.gl().draw_mode(DrawMode::Triangles);
        self.gl().geometry(&vertices, &indices);
    }

    pub fn draw_rectangle_lines(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: Color,
    ) {
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

        self.gl().texture(None);
        self.gl().draw_mode(DrawMode::Triangles);
        self.gl().geometry(&vertices, &indices);
    }

    // pub fn draw_hexagon(
    //     x: f32,
    //     y: f32,
    //     size: f32,
    //     border: f32,
    //     vertical: bool,
    //     border_color: Color,
    //     fill_color: Color,
    // ) {
    //     let rotation = if vertical { 90. } else { 0. };
    //     draw_poly(x, y, 6, size, rotation, fill_color);
    //     if border > 0. {
    //         draw_poly_lines(x, y, 6, size, rotation, border, border_color);
    //     }
    // }

    pub fn draw_poly(
        &mut self,
        x: f32,
        y: f32,
        sides: u8,
        radius: f32,
        rotation: f32,
        color: Color,
    ) {
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

        self.gl().texture(None);
        self.gl().draw_mode(DrawMode::Triangles);
        self.gl().geometry(&vertices, &indices);
    }

    // pub fn draw_poly_lines(
    //     x: f32,
    //     y: f32,
    //     sides: u8,
    //     radius: f32,
    //     rotation: f32,
    //     thickness: f32,
    //     color: Color,
    // ) {
    //     let rot = rotation.to_radians();

    //     for i in 0..sides {
    //         let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
    //         let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

    //         let p0 = vec2(x + radius * rx, y + radius * ry);

    //         let rx = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
    //         let ry = ((i + 1) as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

    //         let p1 = vec2(x + radius * rx, y + radius * ry);

    //         draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
    //     }
    // }

    pub fn draw_circle(&mut self, x: f32, y: f32, r: f32, color: Color) {
        self.draw_poly(x, y, 20, r, 0., color);
    }

    // pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    //     draw_poly_lines(x, y, 20, r, 0., thickness, color);
    // }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
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

        self.gl().texture(None);
        self.gl().draw_mode(DrawMode::Triangles);
        self.gl().geometry(
            &[
                Vertex::new(x1 + tx, y1 + ty, 0., 0., 0., color),
                Vertex::new(x1 - tx, y1 - ty, 0., 0., 0., color),
                Vertex::new(x2 + tx, y2 + ty, 0., 0., 0., color),
                Vertex::new(x2 - tx, y2 - ty, 0., 0., 0., color),
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }
}

pub struct ShapeBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    rotation: f32,
}
impl ShapeBuilder {
    pub fn circle(pos: Vec2, radius: f32, color: Color) -> ShapeBuilder {
        let sides = 50;
        let mut vertices = Vec::<Vertex>::with_capacity(sides as usize + 2);
        let mut indices = Vec::<u16>::with_capacity(sides as usize * 3);

        let rot = 0.0; //0.0.to_radians();
        vertices.push(Vertex::new(pos.x, pos.y, 0., 0., 0., color));
        for i in 0..sides + 1 {
            let rx = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).cos();
            let ry = (i as f32 / sides as f32 * std::f32::consts::PI * 2. + rot).sin();

            let vertex = Vertex::new(pos.x + radius * rx, pos.y + radius * ry, 0., rx, ry, color);

            vertices.push(vertex);

            if i != sides {
                indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
            }
        }

        ShapeBuilder {
            vertices,
            indices,
            rotation: 0.0,
        }
    }

    pub fn line(start: Vec2, end: Vec2, thickness: f32, color: Color) -> ShapeBuilder {
        let (x1, y1) = start.into();
        let (x2, y2) = end.into();
        let dx = x2 - x1;
        let dy = y2 - y1;

        // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

        let nx = -dy;
        let ny = dx;

        let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
        if tlen < std::f32::EPSILON {
            // TODO: check if ShapeBuilder {vertices: vec![]} is ok
            panic!();
        }
        let tx = nx / tlen;
        let ty = ny / tlen;

        let vertices = vec![
            Vertex::new(x1 + tx, y1 + ty, 0., 0., 0., color),
            Vertex::new(x1 - tx, y1 - ty, 0., 0., 0., color),
            Vertex::new(x2 + tx, y2 + ty, 0., 0., 0., color),
            Vertex::new(x2 - tx, y2 - ty, 0., 0., 0., color),
        ];
        let indices = vec![0, 1, 2, 2, 1, 3];

        ShapeBuilder {
            vertices,
            indices,
            rotation: 0.0,
        }
    }

    pub fn rectangle(rect: Rect, color: Color) -> ShapeBuilder {
        let Rect { x, y, w, h } = rect;
        #[rustfmt::skip]
        let vertices = vec![
            Vertex::new(x    , y    , 0., 0.0, 0.0, color),
            Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
            Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
            Vertex::new(x    , y + h, 0., 0.0, 1.0, color),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];

        ShapeBuilder {
            vertices,
            indices,
            rotation: 0.0,
        }
    }

    pub fn rotation(self, rotation: f32) -> ShapeBuilder {
        Self { rotation, ..self }
    }

    pub fn draw(self, canvas: &mut SpriteLayer) {
        canvas.gl().texture(None);
        canvas.gl().draw_mode(DrawMode::Triangles);
        canvas.gl().geometry(&self.vertices, &self.indices);
    }
}
