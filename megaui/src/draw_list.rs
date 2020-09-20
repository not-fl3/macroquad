use crate::draw_command::DrawCommand;
use crate::types::{Color, Rect};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub type VertexInterop = ([f32; 3], [f32; 2], [f32; 4]);

impl From<Vertex> for VertexInterop {
    fn from(value: Vertex) -> Self {
        (
            [value.pos[0], value.pos[1], value.pos[2]],
            [value.uv[0], value.uv[1]],
            value.color,
        )
    }
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, color: Color) -> Vertex {
        Vertex {
            pos: [x, y, 0.],
            uv: [u, v],
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

#[derive(Debug)]
pub struct DrawList {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub clipping_zone: Option<Rect>,
    pub texture: Option<u32>,
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList {
            vertices: vec![],
            indices: vec![],
            clipping_zone: None,
            texture: None,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.clipping_zone = None;
    }

    pub fn draw_rectangle_lines(&mut self, rect: Rect, color: Color) {
        let Rect { x, y, w, h } = rect;

        self.draw_rectangle(Rect { x, y, w, h: 1. }, Rect::new(0., 0., 0., 0.), color);
        self.draw_rectangle(
            Rect {
                x: x + w - 1.,
                y: y + 1.,
                w: 1.,
                h: h - 2.,
            },
            Rect::new(0., 0., 0., 0.),
            color,
        );
        self.draw_rectangle(
            Rect {
                x,
                y: y + h - 1.,
                w,
                h: 1.,
            },
            Rect::new(0., 0., 0., 0.),
            color,
        );
        self.draw_rectangle(
            Rect {
                x,
                y: y + 1.,
                w: 1.,
                h: h - 2.,
            },
            Rect::new(0., 0., 0., 0.),
            color,
        );
    }

    fn draw_rectangle(&mut self, rect: Rect, src: Rect, color: Color) {
        let Rect { x, y, w, h } = rect;

        #[rustfmt::skip]
        let vertices = [
            Vertex::new(x    , y    , src.x        , src.y        , color),
            Vertex::new(x + w, y    , src.x + src.w, src.y        , color),
            Vertex::new(x + w, y + h, src.x + src.w, src.y + src.h, color),
            Vertex::new(x    , y + h, src.x        , src.y + src.h, color),
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let indices_offset = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&vertices[..]);
        self.indices
            .extend(indices.iter().map(|i| i + indices_offset));
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let nx = -dy; // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment
        let ny = dx;

        let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
        if tlen < std::f32::EPSILON {
            return;
        }
        let tx = nx / tlen;
        let ty = ny / tlen;

        let vertices = &[
            Vertex::new(x1 + tx, y1 + ty, 0., 0., color),
            Vertex::new(x1 - tx, y1 - ty, 0., 0., color),
            Vertex::new(x2 + tx, y2 + ty, 0., 0., color),
            Vertex::new(x2 - tx, y2 - ty, 0., 0., color),
        ];
        let indices = &[0, 1, 2, 2, 1, 3];

        let indices_offset = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&vertices[..]);
        self.indices
            .extend(indices.iter().map(|i| i + indices_offset));
    }
}

fn get_active_draw_list<'a, 'b>(
    draw_lists: &'a mut Vec<DrawList>,
    command: &'b DrawCommand,
) -> &'a mut DrawList {
    if draw_lists.len() == 0 {
        draw_lists.push(DrawList::new());
    }

    let last = draw_lists.last().unwrap();
    match command {
        DrawCommand::Clip { rect, .. } => {
            if last.clipping_zone != *rect {
                draw_lists.push(DrawList::new())
            }
        }
        DrawCommand::DrawRawTexture { texture, .. } => {
            if last.texture != Some(*texture) {
                draw_lists.push(DrawList::new());
            }
        }
        DrawCommand::DrawCharacter { .. }
        | DrawCommand::DrawLine { .. }
        | DrawCommand::DrawRect { .. } => {
            if last.texture != None {
                draw_lists.push(DrawList::new());
            }
        }
    }
    draw_lists.last_mut().unwrap()
}

pub(crate) fn render_command(draw_lists: &mut Vec<DrawList>, command: DrawCommand) {
    let active_draw_list = get_active_draw_list(draw_lists, &command);

    match command {
        DrawCommand::Clip { rect, .. } => {
            active_draw_list.clipping_zone = rect;
        }
        DrawCommand::DrawRect { rect, fill, stroke } => {
            if let Some(fill) = fill {
                active_draw_list.draw_rectangle(rect, Rect::new(0., 0., 0., 0.), fill);
            }
            if let Some(stroke) = stroke {
                active_draw_list.draw_rectangle_lines(rect, stroke);
            }
        }
        DrawCommand::DrawLine { start, end, color } => {
            active_draw_list.draw_line(start.x, start.y, end.x, end.y, 1., color);
        }
        DrawCommand::DrawCharacter {
            dest,
            source,
            color,
        } => {
            active_draw_list.draw_rectangle(dest, source, color);
        }
        _ => {}
    }
}
