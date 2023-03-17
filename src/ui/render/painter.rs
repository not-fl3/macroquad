//! Resolve high-level drawing primitive + given style into DrawCommand
//! DrawCommand will later rasterized into mesh in mesh_rasterizer.rs

use crate::{
    color::Color,
    math::{vec2, Rect, RectOffset, Vec2},
    text::{
        atlas::{Atlas, SpriteKey},
        Font, TextDimensions,
    },
    texture::Texture2D,
    ui::{style::Style, UiContent},
};

use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ElementState {
    pub focused: bool,
    pub hovered: bool,
    pub clicked: bool,
    pub selected: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum DrawCommand {
    DrawCharacter {
        dest: Rect,
        source: Rect,
        color: Color,
    },
    DrawRect {
        rect: Rect,
        source: Rect,
        fill: Option<Color>,
        stroke: Option<Color>,
    },
    DrawSprite {
        rect: Rect,
        source: Rect,
        color: Color,
        offsets: Option<RectOffset>,
        offsets_uv: Option<RectOffset>,
    },
    DrawTriangle {
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        source: Rect,
        color: Color,
    },
    DrawLine {
        start: Vec2,
        end: Vec2,
        source: Rect,
        color: Color,
    },
    DrawRawTexture {
        rect: Rect,
        texture: Texture2D,
    },
    Clip {
        rect: Option<Rect>,
    },
}

impl DrawCommand {
    pub fn offset(&self, offset: Vec2) -> DrawCommand {
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
            DrawCommand::DrawRawTexture { rect, texture } => DrawCommand::DrawRawTexture {
                rect: rect.offset(offset),
                texture,
            },
            DrawCommand::DrawRect {
                rect,
                source,
                fill,
                stroke,
            } => DrawCommand::DrawRect {
                rect: rect.offset(offset),
                source,
                fill,
                stroke,
            },
            DrawCommand::DrawSprite {
                rect,
                source,
                color,
                offsets,
                offsets_uv,
            } => DrawCommand::DrawSprite {
                rect: rect.offset(offset),
                source,
                color,
                offsets,
                offsets_uv,
            },
            DrawCommand::DrawLine {
                start,
                end,
                source,
                color,
            } => DrawCommand::DrawLine {
                start: start + offset,
                end: end + offset,
                source,
                color,
            },
            DrawCommand::DrawTriangle {
                p0,
                p1,
                p2,
                source,
                color,
            } => DrawCommand::DrawTriangle {
                p0: p0 + offset,
                p1: p1 + offset,
                p2: p2 + offset,
                source,
                color,
            },
            DrawCommand::Clip { rect } => DrawCommand::Clip {
                rect: rect.map(|rect| rect.offset(offset)),
            },
        }
    }

    pub(crate) fn estimate_triangles_budget(&self) -> (usize, usize) {
        match self {
            DrawCommand::DrawCharacter { .. } => (10, 10),
            DrawCommand::DrawRawTexture { .. } => (10, 10),
            DrawCommand::DrawRect { .. } => (10, 10),
            DrawCommand::DrawLine { .. } => (10, 10),
            DrawCommand::DrawTriangle { .. } => (10, 10),
            _ => (0, 0),
        }
    }
}

pub(crate) struct Painter {
    pub commands: Vec<DrawCommand>,
    pub clipping_zone: Option<Rect>,
    font_atlas: Arc<Mutex<Atlas>>,
}

impl Painter {
    pub fn new(font_atlas: Arc<Mutex<Atlas>>) -> Painter {
        Painter {
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
    pub fn character_advance(&self, character: char, font: &Font, font_size: u16) -> f32 {
        if let Some(font_data) = font.get(character, font_size) {
            return font_data.advance;
        }

        0.
    }

    pub fn content_with_margins_size(&self, style: &Style, content: &UiContent) -> Vec2 {
        let font = &mut *style.font.lock().unwrap();
        let font_size = style.font_size;

        let background_margin = style.background_margin.unwrap_or_default();
        let margin = style.margin.unwrap_or_default();

        let size = match content {
            UiContent::Label(label) => {
                let text_measures = self.label_size(&*label, None, font, font_size);
                (text_measures.width, font_size as f32)
            }
            UiContent::Texture(texture) => (texture.width(), texture.height()),
        };

        vec2(size.0, size.1)
            + Vec2::new(
                margin.left + margin.right + background_margin.left + background_margin.right,
                margin.top + margin.bottom + background_margin.top + background_margin.bottom,
            )
    }

    pub fn draw_element_background(
        &mut self,
        style: &Style,
        pos: Vec2,
        size: Vec2,
        element_state: ElementState,
    ) {
        let color = style.color(element_state);

        let background_margin = style.background_margin.unwrap_or_default();
        if let Some(background) = style.background_sprite(element_state) {
            self.draw_sprite(
                Rect::new(pos.x, pos.y, size.x, size.y),
                background,
                color,
                Some(background_margin),
            );
        } else {
            self.draw_rect(Rect::new(pos.x, pos.y, size.x, size.y), None, color);
        }
    }

    // mostly legacy, technically everything should use `draw_element_content`
    // but draw_element_label had a slightly different margins resolver, so..
    pub fn draw_element_label(
        &mut self,
        style: &Style,
        pos: Vec2,
        label: &str,
        element_state: ElementState,
    ) {
        let font = &mut *style.font.lock().unwrap();
        let font_size = style.font_size;

        let text_measures = self.label_size(label, None, font, font_size);
        let background_margin = style.background_margin.unwrap_or_default();
        let margin = style.margin.unwrap_or_default();

        let top_coord = (font_size as f32) / 2. - (text_measures.height / 2.).trunc()
            + margin.top
            + background_margin.top;

        self.draw_label(
            label,
            pos + Vec2::new(
                margin.left + background_margin.left,
                top_coord + text_measures.offset_y,
            ),
            Some(style.text_color(element_state)),
            font,
            font_size,
        );
    }

    pub fn draw_element_content(
        &mut self,
        style: &Style,
        element_pos: Vec2,
        element_size: Vec2,
        content: &UiContent,
        element_state: ElementState,
    ) {
        match content {
            UiContent::Label(data) => {
                let font = &mut *style.font.lock().unwrap();
                let font_size = style.font_size;
                let text_color = style.text_color(element_state);
                let text_measures = self.label_size(data, None, font, font_size);

                let left_coord = (element_size.x - text_measures.width) / 2.;
                let top_coord =
                    element_size.y / 2. - text_measures.height / 2. + text_measures.offset_y;

                self.draw_label(
                    &*data,
                    element_pos + Vec2::new(left_coord, top_coord),
                    Some(text_color),
                    font,
                    font_size,
                );
            }
            UiContent::Texture(texture) => {
                let background_margin = style.background_margin.unwrap_or_default();
                let margin = style.margin.unwrap_or_default();

                let top_coord = margin.top + background_margin.top;

                let pos = element_pos + Vec2::new(margin.left + background_margin.left, top_coord);
                let size = element_size
                    - vec2(
                        background_margin.left + background_margin.right,
                        background_margin.top + background_margin.bottom,
                    )
                    - vec2(margin.left + margin.right, margin.top + margin.bottom);

                self.draw_raw_texture(Rect::new(pos.x, pos.y, size.x, size.y), texture);
            }
        }
    }

    pub fn label_size(
        &self,
        label: &str,
        _multiline: Option<f32>,
        font: &mut Font,
        font_size: u16,
    ) -> TextDimensions {
        font.measure_text(label, font_size, 1.0, 1.0)
    }

    /// If character is in font atlas - will return x advance from position to potential next character position
    pub fn draw_character(
        &mut self,
        character: char,
        position: Vec2,
        color: Color,
        font: &mut Font,
        font_size: u16,
    ) -> Option<f32> {
        if font.get(character, font_size).is_none() {
            font.cache_glyph(character, font_size);
        }

        if let Some(font_data) = font.get(character, font_size) {
            let glyph = self
                .font_atlas
                .lock()
                .unwrap()
                .get(font_data.sprite)
                .unwrap();
            let left_coord = font_data.offset_x as f32;
            let top_coord = -glyph.rect.h - font_data.offset_y as f32;
            let dest = Rect::new(
                left_coord + position.x,
                top_coord + position.y,
                glyph.rect.w,
                glyph.rect.h,
            );
            if self
                .clipping_zone
                .map_or(false, |clip| !clip.overlaps(&dest))
            {
                let advance = font_data.advance;
                return Some(advance);
            }

            let source = self
                .font_atlas
                .lock()
                .unwrap()
                .get_uv_rect(font_data.sprite);

            if let Some(source) = source {
                let cmd = DrawCommand::DrawCharacter {
                    dest,
                    source,
                    color,
                };
                self.add_command(cmd);
                return Some(font_data.advance);
            }
        }

        None
    }

    pub fn draw_label<T: Into<LabelParams>>(
        &mut self,
        label: &str,
        position: Vec2,
        params: T,
        font: &mut Font,
        font_size: u16,
    ) {
        if self.clipping_zone.map_or(false, |clip| {
            !clip.overlaps(&Rect::new(position.x - 150., position.y - 25., 200., 50.))
        }) {
            return;
        }

        let params = params.into();

        let mut total_width = 0.;
        let position = vec2(position.x.trunc(), position.y.trunc());
        for character in label.chars() {
            if let Some(advance) = self.draw_character(
                character,
                position + Vec2::new(total_width, 0.),
                params.color,
                font,
                font_size,
            ) {
                total_width += advance;
            }
        }
    }

    pub fn draw_raw_texture(&mut self, rect: Rect, texture: &Texture2D) {
        if self
            .clipping_zone
            .map_or(false, |clip| !clip.overlaps(&rect))
        {
            return;
        }

        self.add_command(DrawCommand::DrawRawTexture {
            rect,
            texture: texture.clone(),
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

        let source = self
            .font_atlas
            .lock()
            .unwrap()
            .get_uv_rect(SpriteKey::Id(0))
            .unwrap();
        self.add_command(DrawCommand::DrawRect {
            rect,
            source,
            stroke: stroke.into(),
            fill: fill.into(),
        })
    }

    pub fn draw_sprite(
        &mut self,
        rect: Rect,
        sprite: SpriteKey,
        color: Color,
        margin: Option<RectOffset>,
    ) {
        if self
            .clipping_zone
            .map_or(false, |clip| !clip.overlaps(&rect))
        {
            return;
        }

        let atlas = self.font_atlas.lock().unwrap();
        let source_uv = atlas.get_uv_rect(sprite).unwrap();
        let (w, h) = (atlas.width(), atlas.height());
        drop(atlas);
        self.add_command(DrawCommand::DrawSprite {
            rect,
            source: source_uv,
            color,
            offsets: margin,
            offsets_uv: margin.map(|margin| RectOffset {
                left: margin.left / w as f32,
                right: margin.right / w as f32,
                top: margin.top / h as f32,
                bottom: margin.bottom / h as f32,
            }),
        })
    }

    #[allow(dead_code)]
    pub fn draw_triangle<T>(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, color: T)
    where
        T: Into<Color>,
    {
        if self.clipping_zone.map_or(false, |clip| {
            !clip.contains(p0) && !clip.contains(p1) && !clip.contains(p2)
        }) {
            return;
        }

        let source = self
            .font_atlas
            .lock()
            .unwrap()
            .get_uv_rect(SpriteKey::Id(0))
            .unwrap();

        self.add_command(DrawCommand::DrawTriangle {
            p0,
            p1,
            p2,
            source,
            color: color.into(),
        })
    }

    pub fn draw_line<T: Into<Color>>(&mut self, start: Vec2, end: Vec2, color: T) {
        if self
            .clipping_zone
            .map_or(false, |clip| !clip.contains(start) && !clip.contains(end))
        {
            return;
        }

        let source = self
            .font_atlas
            .lock()
            .unwrap()
            .get_uv_rect(SpriteKey::Id(0))
            .unwrap();
        self.add_command(DrawCommand::DrawLine {
            start,
            end,
            source,
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

        let scaled_clipping_zone = self.clipping_zone.map(|rect| {
            let dpi = miniquad::window::dpi_scale();
            Rect::new(rect.x * dpi, rect.y * dpi, rect.w * dpi, rect.h * dpi)
        });
        self.add_command(DrawCommand::Clip { rect: scaled_clipping_zone });
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Alignment {
    Left,
    Center,
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::Left
    }
}

#[derive(Clone, Debug)]
pub struct LabelParams {
    pub color: Color,
    pub alignment: Alignment,
}

impl Default for LabelParams {
    fn default() -> LabelParams {
        LabelParams {
            color: Color::new(0., 0., 0., 1.),
            alignment: Alignment::default(),
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
impl From<(Color, Alignment)> for LabelParams {
    fn from((color, alignment): (Color, Alignment)) -> LabelParams {
        LabelParams { color, alignment }
    }
}
