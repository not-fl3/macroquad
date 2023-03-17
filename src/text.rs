//! Functions to load fonts and draw text.

use std::collections::HashMap;

use crate::{
    color::Color,
    get_context, get_quad_context,
    math::{vec3, Rect},
    texture::{Image, TextureHandle},
    Error,
};

use crate::color::WHITE;
use glam::vec2;

use std::sync::{Arc, Mutex};
pub(crate) mod atlas;

use atlas::{Atlas, SpriteKey};

#[derive(Debug, Clone)]
pub(crate) struct CharacterInfo {
    pub offset_x: i32,
    pub offset_y: i32,
    pub advance: f32,
    pub sprite: SpriteKey,
}

/// TTF font loaded to GPU
#[derive(Clone)]
pub struct Font {
    font: fontdue::Font,
    atlas: Arc<Mutex<Atlas>>,
    characters: Arc<Mutex<HashMap<(char, u16), CharacterInfo>>>,
}

fn require_fn_to_be_send() {
    fn require_send<T: Send>() {}
    require_send::<Font>();
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("font", &"fontdue::Font")
            .finish()
    }
}

impl Font {
    pub(crate) fn load_from_bytes(atlas: Arc<Mutex<Atlas>>, bytes: &[u8]) -> Result<Font, Error> {
        Ok(Font {
            font: fontdue::Font::from_bytes(&bytes[..], fontdue::FontSettings::default())?,
            characters: Arc::new(Mutex::new(HashMap::new())),
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

    pub(crate) fn cache_glyph(&self, character: char, size: u16) {
        if self
            .characters
            .lock()
            .unwrap()
            .contains_key(&(character, size))
        {
            return;
        }

        let (metrics, bitmap) = self.font.rasterize(character, size as f32);

        if metrics.advance_height != 0.0 {
            panic!("Vertical fonts are not supported");
        }

        let (width, height) = (metrics.width as u16, metrics.height as u16);

        let sprite = self.atlas.lock().unwrap().new_unique_id();
        self.atlas.lock().unwrap().cache_sprite(
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

        self.characters
            .lock()
            .unwrap()
            .insert((character, size), character_info);
    }

    pub(crate) fn get(&self, character: char, size: u16) -> Option<CharacterInfo> {
        self.characters
            .lock()
            .unwrap()
            .get(&(character, size))
            .cloned()
    }

    pub(crate) fn measure_text(
        &self,
        text: &str,
        font_size: u16,
        font_scale_x: f32,
        font_scale_y: f32,
    ) -> TextDimensions {
        let dpi_scaling = miniquad::window::dpi_scale();
        let font_size = (font_size as f32 * dpi_scaling).ceil() as u16;

        for character in text.chars() {
            if self
                .characters
                .lock()
                .unwrap()
                .contains_key(&(character, font_size))
                == false
            {
                self.cache_glyph(character, font_size);
            }
        }

        let mut width = 0.;
        let mut min_y = std::f32::MAX;
        let mut max_y = -std::f32::MAX;

        let atlas = self.atlas.lock().unwrap();

        for character in text.chars() {
            if let Some(font_data) = self.characters.lock().unwrap().get(&(character, font_size)) {
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
            width: width / dpi_scaling,
            height: height / dpi_scaling,
            offset_y: max_y / dpi_scaling,
        }
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
        for character in characters {
            self.cache_glyph(*character, size);
        }
    }

    /// Sets the [FilterMode](https://docs.rs/miniquad/latest/miniquad/graphics/enum.FilterMode.html#) of this font's texture atlas.
    ///
    /// Use Nearest if you need integer-ratio scaling for pixel art, for example.
    ///
    /// # Example
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// let font = Font::default();
    /// font.set_filter(FilterMode::Linear);
    /// # }
    /// ```
    pub fn set_filter(&mut self, filter_mode: miniquad::FilterMode) {
        self.atlas.lock().unwrap().set_filter(filter_mode);
    }

    // pub fn texture(&self) -> Texture2D {
    //     let font = get_context().fonts_storage.get_font(*self);

    //     font.font_texture
    // }
}

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone)]
pub struct TextParams<'a> {
    pub font: Option<&'a Font>,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters may be blurry
    pub font_scale: f32,
    /// Font X axis would be scaled by font_scale * font_scale_aspect
    /// and Y axis would be scaled by font_scale
    /// Default is 1.0
    pub font_scale_aspect: f32,
    /// Text rotation in radian
    /// Default is 0.0
    pub rotation: f32,
    pub color: Color,
}

impl<'a> Default for TextParams<'a> {
    fn default() -> TextParams<'a> {
        TextParams {
            font: None,
            font_size: 20,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            color: WHITE,
            rotation: 0.0,
        }
    }
}

/// Load font from file with "path"
pub async fn load_ttf_font(path: &str) -> Result<Font, Error> {
    let bytes = crate::file::load_file(path)
        .await
        .map_err(|_| Error::FontError("The Font file couldn't be loaded"))?;

    load_ttf_font_from_bytes(&bytes[..])
}

/// Load font from bytes array, may be use in combination with include_bytes!
/// ```ignore
/// let font = load_ttf_font_from_bytes(include_bytes!("font.ttf"));
/// ```
pub fn load_ttf_font_from_bytes(bytes: &[u8]) -> Result<Font, Error> {
    let context = get_context();
    let atlas = Arc::new(Mutex::new(Atlas::new(
        get_quad_context(),
        miniquad::FilterMode::Linear,
    )));

    let mut font = Font::load_from_bytes(atlas.clone(), bytes)?;

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
    let font = params
        .font
        .unwrap_or(&get_context().fonts_storage.default_font);

    let font_scale_x = params.font_scale * params.font_scale_aspect;
    let font_scale_y = params.font_scale;
    let dpi_scaling = miniquad::window::dpi_scale();

    let font_size = (params.font_size as f32 * dpi_scaling).ceil() as u16;

    let mut total_width = 0.;
    for character in text.chars() {
        if !font
            .characters
            .lock()
            .unwrap()
            .contains_key(&(character, font_size))
        {
            font.cache_glyph(character, font_size);
        }
        let mut atlas = font.atlas.lock().unwrap();
        let font_data = &font.characters.lock().unwrap()[&(character, font_size)];
        let glyph = atlas.get(font_data.sprite).unwrap().rect;
        let angle_rad = params.rotation;
        let left_coord = (font_data.offset_x as f32 * font_scale_x + total_width) * angle_rad.cos()
            + (glyph.h as f32 * font_scale_y + font_data.offset_y as f32 * font_scale_y)
                * angle_rad.sin();
        let top_coord = (font_data.offset_x as f32 * font_scale_x + total_width) * angle_rad.sin()
            + (0.0 - glyph.h as f32 * font_scale_y - font_data.offset_y as f32 * font_scale_y)
                * angle_rad.cos();

        total_width += font_data.advance * font_scale_x;

        let dest = Rect::new(
            left_coord / dpi_scaling as f32 + x,
            top_coord / dpi_scaling as f32 + y,
            glyph.w as f32 / dpi_scaling as f32 * font_scale_x,
            glyph.h as f32 / dpi_scaling as f32 * font_scale_y,
        );

        let source = Rect::new(
            glyph.x as f32,
            glyph.y as f32,
            glyph.w as f32,
            glyph.h as f32,
        );

        crate::texture::draw_texture_ex(
            &crate::texture::Texture2D {
                texture: TextureHandle::Unmanaged(atlas.texture()),
            },
            dest.x,
            dest.y,
            params.color,
            crate::texture::DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(source),
                rotation: angle_rad,
                pivot: Option::Some(vec2(dest.x, dest.y)),
                ..Default::default()
            },
        );
    }
}

/// Get the text center.
pub fn get_text_center(
    text: &str,
    font: Option<&Font>,
    font_size: u16,
    font_scale: f32,
    rotation: f32,
) -> crate::Vec2 {
    let measure = measure_text(text, font, font_size, font_scale);

    let x_center = measure.width / 2.0 * rotation.cos() + measure.height / 2.0 * rotation.sin();
    let y_center = measure.width / 2.0 * rotation.sin() - measure.height / 2.0 * rotation.cos();

    crate::Vec2::new(x_center, y_center)
}

/// World space dimensions of the text, measured by "measure_text" function
#[derive(Debug, Clone, Copy)]
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
    font: Option<&Font>,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    let font = font.unwrap_or_else(|| &get_context().fonts_storage.default_font);

    font.measure_text(text, font_size, font_scale, font_scale)
}

pub(crate) struct FontsStorage {
    default_font: Font,
}

impl FontsStorage {
    pub(crate) fn new(ctx: &mut dyn miniquad::RenderingBackend) -> FontsStorage {
        let atlas = Arc::new(Mutex::new(Atlas::new(ctx, miniquad::FilterMode::Linear)));

        let default_font = Font::load_from_bytes(atlas, include_bytes!("ProggyClean.ttf")).unwrap();
        FontsStorage { default_font }
    }
}

/// From given font size in world space gives
/// (font_size, font_scale and font_aspect) params to make rasterized font
/// looks good in currently active camera
pub fn camera_font_scale(world_font_size: f32) -> (u16, f32, f32) {
    let context = get_context();
    let (scr_w, scr_h) = miniquad::window::screen_size();
    let cam_space = context
        .projection_matrix()
        .inverse()
        .transform_vector3(vec3(2., 2., 0.));
    let (cam_w, cam_h) = (cam_space.x.abs(), cam_space.y.abs());

    let screen_font_size = world_font_size * scr_h / cam_h;

    let font_size = screen_font_size as u16;

    (font_size, cam_h / scr_h, scr_h / scr_w * cam_w / cam_h)
}
