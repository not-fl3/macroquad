use crate::{Color, Rect, Vector2};

use miniquad_text_rusttype::FontAtlas;

use std::rc::Rc;

trait Wtf: std::any::Any + Clone {
}

#[derive(Debug, Clone)]
pub(crate) enum DrawCommand {
    DrawCharacter {
        dest: Rect,
        source: Rect,
        color: Color,
    },
    DrawRect {
        rect: Rect,
        stroke: Option<Color>,
        fill: Option<Color>,
    },
    DrawTriangle {
        p0: Vector2,
        p1: Vector2,
        p2: Vector2,
        color: Color,
    },
    DrawLine {
        start: Vector2,
        end: Vector2,
        color: Color,
    },
    DrawRawTexture {
        rect: Rect,
        texture: u32,
    },
    Clip {
        rect: Option<Rect>,
    },
}

impl DrawCommand {
    pub fn offset(&self, offset: Vector2) -> DrawCommand {
        match self.clone() {
            DrawCommand::DrawCharacter {
                dest,
                source,
                color,
            } => DrawCommand::DrawCharacter {
                dest: dest.offset(offset),
                source,
                color,
            },
            DrawCommand::DrawRawTexture {
                rect,
                texture,
            } => DrawCommand::DrawRawTexture {
                rect: rect.offset(offset),
                texture,
            },
            DrawCommand::DrawRect { rect, stroke, fill } => DrawCommand::DrawRect {
                rect: rect.offset(offset),
                stroke,
                fill,
            },
            DrawCommand::DrawLine { start, end, color } => DrawCommand::DrawLine {
                start: start + offset,
                end: end + offset,
                color,
            },
            DrawCommand::DrawTriangle { p0, p1, p2, color } => DrawCommand::DrawTriangle {
                p0: p0 + offset,
                p1: p1 + offset,
                p2: p2 + offset,
                color,
            },
            DrawCommand::Clip { rect } => DrawCommand::Clip {
                rect: rect.map(|rect| rect.offset(offset)),
            },
        }
    }
}

pub(crate) struct CommandsList {
    pub commands: Vec<DrawCommand>,
    pub clipping_zone: Option<Rect>,
    font_atlas: Rc<FontAtlas>,
}

impl CommandsList {
    pub fn new(font_atlas: Rc<FontAtlas>) -> CommandsList {
        CommandsList {
            commands: vec![],
            clipping_zone: None,
            font_atlas,
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.clipping_zone = None;
    }

    fn add_command(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// calculate character horizontal size,
    /// usually used as an advance between current cursor position
    /// and next potential character
    pub fn character_advance(&self, character: char) -> f32 {
        if let Some(font_data) = self.font_atlas.character_infos.get(&character) {
            let font_data = font_data.scale(self.font_atlas.font_size as f32);
            let advance = font_data.left_padding + font_data.size.0 + font_data.right_padding;

            return advance;
        }

        0.
    }

    pub fn label_size(&self, label: &str, multiline: Option<f32>) -> Vector2 {
        let width = label.split('\n').fold(0.0f32, |max_width, line| {
            max_width.max(line.chars().map(|c| self.character_advance(c)).sum::<f32>())
        });
        let height = multiline.map_or(14., |line_height| {
            line_height * label.split('\n').count() as f32
        });

        Vector2::new(width, height)
    }

    /// If character is in font atlas - will return x advance from position to potential next character position
    pub fn draw_character(
        &mut self,
        character: char,
        position: Vector2,
        color: Color,
    ) -> Option<f32> {
        if let Some(font_data) = self.font_atlas.character_infos.get(&character) {
            let font_data = font_data.scale(self.font_atlas.font_size as f32);

            let left_coord = font_data.left_padding;
            // 4.0 cames from lack of understanding of how ttf works
            // with 4.0 top_coord is a top_coord of any buttons, wich makes a character be drawen like:
            // (x, y).....................(x + advance, y)
            // ...........................
            // (x, y + self.font_size.y)..(x + advance, y + _)
            let top_coord = self.font_atlas.font_size as f32 - font_data.height_over_line - 4.0;

            let cmd = DrawCommand::DrawCharacter {
                dest: Rect::new(
                    left_coord + position.x,
                    top_coord + position.y,
                    font_data.size.0,
                    font_data.size.1,
                ),
                source: Rect::new(
                    font_data.tex_coords.0,
                    font_data.tex_coords.1,
                    font_data.tex_size.0,
                    font_data.tex_size.1,
                ),
                color: color,
            };
            self.add_command(cmd);

            let advance = font_data.left_padding + font_data.size.0 + font_data.right_padding;
            Some(advance)
        } else {
            None
        }
    }

    pub fn draw_label<T: Into<LabelParams>>(&mut self, label: &str, position: Vector2, params: T) {
        if self.clipping_zone.map_or(false, |clip| {
            !clip.overlaps(&Rect::new(position.x - 150., position.y - 25., 200., 50.))
        }) {
            return;
        }

        let params = params.into();

        let mut total_width = 0.;
        for character in label.chars() {
            if let Some(advance) = self.draw_character(
                character,
                position + Vector2::new(total_width, 0.),
                params.color,
            ) {
                total_width += advance;
            }
        }
    }

    pub fn draw_raw_texture(&mut self, rect: Rect, texture: u32) {
        if self.clipping_zone.map_or(false, |clip| {
            !clip.overlaps(&rect)
        }) {
            return;
        }

        self.add_command(DrawCommand::DrawRawTexture {
            rect,
            texture,
        })
    }

    pub fn draw_rect<S, T>(&mut self, rect: Rect, stroke: S, fill: T)
    where
        S: Into<Option<Color>>,
        T: Into<Option<Color>>,
    {
        if self
            .clipping_zone
            .map_or(false, |clip| !clip.overlaps(&rect))
        {
            return;
        }

        self.add_command(DrawCommand::DrawRect {
            rect,
            stroke: stroke.into(),
            fill: fill.into(),
        })
    }

    pub fn draw_triangle<T>(&mut self, p0: Vector2, p1: Vector2, p2: Vector2, color: T)
    where
        T: Into<Color>,
    {
        if self.clipping_zone.map_or(false, |clip| {
            !clip.contains(p0) && !clip.contains(p1) && !clip.contains(p2)
        }) {
            return;
        }

        self.add_command(DrawCommand::DrawTriangle {
            p0,
            p1,
            p2,
            color: color.into(),
        })
    }

    pub fn draw_line<T: Into<Color>>(&mut self, start: Vector2, end: Vector2, color: T) {
        if self
            .clipping_zone
            .map_or(false, |clip| !clip.contains(start) && !clip.contains(end))
        {
            return;
        }

        self.add_command(DrawCommand::DrawLine {
            start,
            end,
            color: color.into(),
        });
    }

    #[rustfmt::skip]
    pub fn clip<T: Into<Option<Rect>>>(&mut self, rect: T) {
        let rect = rect.into();

        self.clipping_zone = if let Some(rect) = rect {
            Some(self.clipping_zone.and_then(|old_rect| old_rect.intersect(rect)).unwrap_or(rect))
        } else {
            None
        };


        self.add_command(DrawCommand::Clip { rect: self.clipping_zone });
    }
}

#[derive(Clone, Debug)]
pub enum Aligment {
    Left,
    Center,
}

impl Default for Aligment {
    fn default() -> Aligment {
        Aligment::Left
    }
}

#[derive(Clone, Debug)]
pub struct LabelParams {
    pub color: Color,
    pub aligment: Aligment,
}

impl Default for LabelParams {
    fn default() -> LabelParams {
        LabelParams {
            color: Color::new(0., 0., 0., 1.),
            aligment: Aligment::default(),
        }
    }
}

impl From<Option<Color>> for LabelParams {
    fn from(color: Option<Color>) -> LabelParams {
        LabelParams {
            color: color.unwrap_or(Color::new(0., 0., 0., 1.)),
            ..Default::default()
        }
    }
}
impl From<Color> for LabelParams {
    fn from(color: Color) -> LabelParams {
        LabelParams {
            color,
            ..Default::default()
        }
    }
}
impl From<(Color, Aligment)> for LabelParams {
    fn from((color, aligment): (Color, Aligment)) -> LabelParams {
        LabelParams { color, aligment }
    }
}
