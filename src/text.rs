//! Functions to load fonts and draw text.

use std::collections::HashMap;

use crate::{
    color::Color,
    get_context,
    math::{vec3, Rect},
    texture::Image,
};

use crate::color::WHITE;
use glam::vec2;

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) mod atlas;

use atlas::Atlas;

#[derive(Debug)]
pub(crate) struct CharacterInfo {
    pub offset_x: i32,
    pub offset_y: i32,
    pub advance: f32,
    pub sprite: u64,
}

pub(crate) struct FontInternal {
    font: fontdue::Font,
    atlas: Rc<RefCell<Atlas>>,
    characters: HashMap<(char, u16), CharacterInfo>,
}

impl std::fmt::Debug for FontInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("font", &"fontdue::Font")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct FontError(pub &'static str);

impl From<&'static str> for FontError {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "font error: {}", self.0)
    }
}

impl std::error::Error for FontError {}

impl FontInternal {
    pub(crate) fn load_from_bytes(
        atlas: Rc<RefCell<Atlas>>,
        bytes: &[u8],
    ) -> Result<FontInternal, FontError> {
        Ok(FontInternal {
            font: fontdue::Font::from_bytes(&bytes[..], fontdue::FontSettings::default())?,
            characters: HashMap::new(),
            atlas,
        })
    }

    pub(crate) fn ascent(&self, font_size: f32) -> f32 {
        self.font.horizontal_line_metrics(font_size).unwrap().ascent
    }

    pub(crate) fn descent(&self, font_size: f32) -> f32 {
        self.font
            .horizontal_line_metrics(font_size)
            .unwrap()
            .descent
    }

    pub(crate) fn cache_glyph(&mut self, character: char, size: u16) {
        if self.characters.contains_key(&(character, size)) {
            return;
        }

        let (metrics, bitmap) = self.font.rasterize(character, size as f32);

        if metrics.advance_height != 0.0 {
            panic!("Vertical fonts are not supported");
        }

        let (width, height) = (metrics.width as u16, metrics.height as u16);

        let sprite = self.atlas.borrow_mut().new_unique_id();
        self.atlas.borrow_mut().cache_sprite(
            sprite,
            Image {
                bytes: bitmap
                    .iter()
                    .flat_map(|coverage| vec![255, 255, 255, *coverage])
                    .collect(),
                width,
                height,
            },
        );
        let advance = metrics.advance_width;

        let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);

        let character_info = CharacterInfo {
            advance,
            offset_x,
            offset_y,
            sprite,
        };

        self.characters.insert((character, size), character_info);
    }

    pub(crate) fn get(&self, character: char, size: u16) -> Option<&CharacterInfo> {
        self.characters.get(&(character, size))
    }

    pub(crate) fn measure_text(
        &mut self,
        text: &str,
        font_size: u16,
        font_scale_x: f32,
        font_scale_y: f32,
    ) -> TextDimensions {
        for character in text.chars() {
            if !self.characters.contains_key(&(character, font_size)) {
                self.cache_glyph(character, font_size);
            }
        }

        let mut width = 0.;
        let mut min_y = std::f32::MAX;
        let mut max_y = -std::f32::MAX;

        let atlas = self.atlas.borrow();

        for character in text.chars() {
            if let Some(font_data) = self.characters.get(&(character, font_size)) {
                let glyph = atlas.get(font_data.sprite).unwrap().rect;
                width += font_data.advance * font_scale_x;

                if min_y > font_data.offset_y as f32 * font_scale_y {
                    min_y = font_data.offset_y as f32 * font_scale_y;
                }
                if max_y < glyph.h as f32 * font_scale_y + font_data.offset_y as f32 * font_scale_y
                {
                    max_y =
                        glyph.h as f32 * font_scale_y + font_data.offset_y as f32 * font_scale_y;
                }
            }
        }

        let height = max_y - min_y;
        TextDimensions {
            width,
            height,
            offset_y: max_y,
        }
    }
}

/// TTF font loaded to GPU
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font(usize);

impl Default for Font {
    fn default() -> Font {
        Font(0)
    }
}

impl Font {
    /// List of ascii characters, may be helpful in combination with "populate_font_cache"
    pub fn ascii_character_list() -> Vec<char> {
        (0..255).filter_map(::std::char::from_u32).collect()
    }

    /// List of latin characters
    pub fn latin_character_list() -> Vec<char> {
        "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM1234567890!@#$%^&*(){}[].,:"
            .chars()
            .collect()
    }

    pub fn populate_font_cache(&self, characters: &[char], size: u16) {
        let font = get_context().fonts_storage.get_font_mut(*self);

        for character in characters {
            font.cache_glyph(*character, size);
        }
    }

    // pub fn texture(&self) -> Texture2D {
    //     let font = get_context().fonts_storage.get_font(*self);

    //     font.font_texture
    // }
}

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone, Copy)]
pub struct TextParams {
    pub font: Font,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters may be blurry
    pub font_scale: f32,
    /// Font X axis would be scaled by font_scale * font_scale_aspect
    /// and Y axis would be scaled by font_scale
    /// Default is 1.0
    pub font_scale_aspect: f32,
    pub color: Color,
    /// Depth to draw this font at.
    /// See `DrawTextureParams` for more abot depth
    pub depth: f32,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: Font(0),
            font_size: 20,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            color: WHITE,
            depth: 0.0,
        }
    }
}

/// Load font from file with "path"
pub async fn load_ttf_font(path: &str) -> Result<Font, FontError> {
    let bytes = crate::file::load_file(path).await.unwrap();

    load_ttf_font_from_bytes(&bytes[..])
}

/// Load font from bytes array, may be use in combination with include_bytes!
/// ```ignore
/// let font = load_ttf_font_from_bytes(include_bytes!("font.ttf"));
/// ```
pub fn load_ttf_font_from_bytes(bytes: &[u8]) -> Result<Font, FontError> {
    let context = get_context();
    let atlas = Rc::new(RefCell::new(Atlas::new(
        &mut get_context().quad_context,
        miniquad::FilterMode::Linear,
    )));

    let font = context
        .fonts_storage
        .make_font(FontInternal::load_from_bytes(atlas.clone(), bytes)?);

    font.populate_font_cache(&Font::ascii_character_list(), 15);

    Ok(font)
}

/// Draw text with given font_size
pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            font_scale: 1.0,
            color,
            ..Default::default()
        },
    )
}

/// Draw text with custom params such as font, font size and font scale.
pub fn draw_text_ex(text: &str, x: f32, y: f32, params: TextParams) {
    let font = get_context().fonts_storage.get_font_mut(params.font);

    let font_scale_x = params.font_scale * params.font_scale_aspect;
    let font_scale_y = params.font_scale;

    let mut total_width = 0.;
    for character in text.chars() {
        if !font.characters.contains_key(&(character, params.font_size)) {
            font.cache_glyph(character, params.font_size);
        }
        let mut atlas = font.atlas.borrow_mut();
        let font_data = &font.characters[&(character, params.font_size)];
        let glyph = atlas.get(font_data.sprite).unwrap().rect;
        let left_coord = font_data.offset_x as f32 * font_scale_x + total_width;
        let top_coord =
            0.0 - glyph.h as f32 * font_scale_y - font_data.offset_y as f32 * font_scale_y;

        total_width += font_data.advance * font_scale_x;

        let dest = Rect::new(
            left_coord + x,
            top_coord + y,
            glyph.w as f32 * font_scale_x,
            glyph.h as f32 * font_scale_y,
        );

        let source = Rect::new(
            glyph.x as f32,
            glyph.y as f32,
            glyph.w as f32,
            glyph.h as f32,
        );

        crate::texture::draw_texture_ex(
            atlas.texture(),
            dest.x,
            dest.y,
            params.color,
            crate::texture::DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(source),
                depth: params.depth,
                ..Default::default()
            },
        );
    }
}

/// World space dimensions of the text, measured by "measure_text" function
pub struct TextDimensions {
    /// Distance from very left to very right of the rasterized text
    pub width: f32,
    /// Distance from the bottom to the top of the text.
    pub height: f32,
    /// Height offset from the baseline of the text.
    /// "draw_text(.., X, Y, ..)" will be rendered in a "Rect::new(X, Y - dimensions.offset_y, dimensions.width, dimensions.height)"
    /// For reference check "text_dimensions" example.
    pub offset_y: f32,
}

pub fn measure_text(
    text: &str,
    font: Option<Font>,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    let font = get_context()
        .fonts_storage
        .get_font_mut(font.unwrap_or(Font::default()));

    font.measure_text(text, font_size, font_scale, font_scale)
}

pub(crate) struct FontsStorage {
    fonts: Vec<FontInternal>,
}

impl FontsStorage {
    pub(crate) fn new(ctx: &mut miniquad::Context) -> FontsStorage {
        let atlas = Rc::new(RefCell::new(Atlas::new(ctx, miniquad::FilterMode::Linear)));

        let default_font =
            FontInternal::load_from_bytes(atlas, include_bytes!("ProggyClean.ttf")).unwrap();
        FontsStorage {
            fonts: vec![default_font],
        }
    }

    fn make_font(&mut self, font_internal: FontInternal) -> Font {
        self.fonts.push(font_internal);

        Font(self.fonts.len() - 1)
    }

    fn get_font_mut(&mut self, font: Font) -> &mut FontInternal {
        &mut self.fonts[font.0]
    }
}

/// From given font size in world space gives
/// (font_size, font_scale and font_aspect) params to make rasterized font
/// looks good in currently active camera
pub fn camera_font_scale(world_font_size: f32) -> (u16, f32, f32) {
    let context = get_context();
    let (scr_w, scr_h) = context.quad_context.screen_size();
    let cam_space = context
        .projection_matrix()
        .inverse()
        .transform_vector3(vec3(2., 2., 0.));
    let (cam_w, cam_h) = (cam_space.x.abs(), cam_space.y.abs());

    let screen_font_size = world_font_size * scr_h / cam_h;

    let font_size = screen_font_size as u16;

    (font_size, cam_h / scr_h, scr_h / scr_w * cam_w / cam_h)
}
