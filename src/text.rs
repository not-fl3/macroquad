//! Functions to load fonts and draw text.

use std::collections::HashMap;

use crate::{
    get_context,
    types::{Color, Rect},
};

use glam::vec2;
use quad_gl::Image;
use quad_gl::Texture2D;
use quad_gl::WHITE;

struct CharacterInfo {
    offset_x: i32,
    offset_y: i32,
    advance: f32,

    glyph_x: u32,
    glyph_y: u32,
    glyph_w: u32,
    glyph_h: u32,
}

struct FontInternal {
    font: fontdue::Font,
    font_texture: Texture2D,
    font_image: Image,
    characters: HashMap<(char, u16), CharacterInfo>,
    cursor_x: u16,
    cursor_y: u16,
    max_line_height: u16,
}

impl std::fmt::Debug for FontInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("font", &"fontdue::Font")
            .field("font_texture", &self.font_texture)
            .field("font_image", &"macroquad::Image")
            .finish()
    }
}

impl FontInternal {
    // pixel gap between glyphs in the atlas
    const GAP: u16 = 2;

    fn load_from_bytes(ctx: &mut miniquad::Context, bytes: &[u8]) -> FontInternal {
        let font_image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let font_texture =
            Texture2D::from_rgba8(ctx, font_image.width, font_image.height, &font_image.bytes);

        FontInternal {
            font: fontdue::Font::from_bytes(&bytes[..], fontdue::FontSettings::default()).unwrap(),
            font_image,
            font_texture,
            characters: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            max_line_height: 0,
        }
    }

    fn cache_glyph(&mut self, character: char, size: u16) {
        let (metrics, bitmap) = self.font.rasterize(character, size as f32);

        if metrics.advance_height != 0.0 {
            panic!("Vertical fonts are not supported");
        }

        let (width, height) = (metrics.width, metrics.height);

        let advance = metrics.advance_width;

        let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);
        let x = if self.cursor_x + (width as u16) < self.font_image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x;
            self.cursor_x += width as u16 + Self::GAP;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };

        let y = self.cursor_y;

        let character_info = CharacterInfo {
            glyph_x: x as _,
            glyph_y: y as _,
            glyph_w: width as _,
            glyph_h: height as _,

            advance,
            offset_x,
            offset_y,
        };

        self.characters.insert((character, size), character_info);

        // texture bounds exceeded
        if self.cursor_y + height as u16 > self.font_image.height {
            // reset glyph cache state
            let characters = self.characters.drain().collect::<Vec<_>>();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.max_line_height = 0;

            // increase font texture size
            self.font_image = Image::gen_image_color(
                self.font_image.width * 2,
                self.font_image.height * 2,
                Color::new(0.0, 0.0, 0.0, 0.0),
            );
            let ctx = &mut get_context().quad_context;
            self.font_texture = Texture2D::from_rgba8(
                ctx,
                self.font_image.width,
                self.font_image.height,
                &self.font_image.bytes[..],
            );

            // recache all previously cached symbols
            for ((character, size), _) in characters {
                self.cache_glyph(character, size);
            }
        } else {
            for j in 0..height {
                for i in 0..width {
                    let coverage = bitmap[j * width + i] as f32 / 255.0;
                    self.font_image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        Color::new(1.0, 1.0, 1.0, coverage),
                    );
                }
            }
            let ctx = &mut get_context().quad_context;

            self.font_texture.update(ctx, &self.font_image);
        }
    }
}

/// TTF font loaded to GPU
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font(usize);

impl Font {
    /// List of ascii characters, may be helpfull in combination with "populate_font_cache"
    pub fn ascii_character_list() -> Vec<char> {
        (0..255).filter_map(::std::char::from_u32).collect()
    }

    pub fn populate_font_cache(&self, characters: &[char], size: u16) {
        let font = get_context().fonts_storage.get_font_mut(*self);

        for character in characters {
            font.cache_glyph(*character, size);
        }
    }

    pub fn texture(&self) -> Texture2D {
        let font = get_context().fonts_storage.get_font(*self);

        font.font_texture
    }
}

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone, Copy)]
pub struct TextParams {
    pub font: Font,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters will be blurry
    pub font_scale: f32,
    pub color: Color,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: Font(0),
            font_size: 20,
            font_scale: 1.0,
            color: WHITE,
        }
    }
}

/// Load font from file with "path"   
pub async fn load_ttf_font(path: &str) -> Font {
    let bytes = crate::file::load_file(path).await.unwrap();

    load_ttf_font_from_bytes(&bytes[..])
}

/// Load font from bytes array, may be use in combination with include_bytes!
/// ```ignore
/// let font = load_ttf_font_from_bytes(include_bytes!("font.ttf"));
/// ```
pub fn load_ttf_font_from_bytes(bytes: &[u8]) -> Font {
    let font = get_context()
        .fonts_storage
        .make_font(FontInternal::load_from_bytes(
            &mut get_context().quad_context,
            bytes,
        ));

    font.populate_font_cache(&Font::ascii_character_list(), 15);

    font
}

/// Draw text with given font_size
pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            font_scale: font_size % 1.0,
            color,
            ..Default::default()
        },
    )
}

/// Draw text with custom params such as font, font size and font scale.
pub fn draw_text_ex(text: &str, x: f32, y: f32, params: TextParams) {
    let font = get_context().fonts_storage.get_font_mut(params.font);

    let mut total_width = 0.;
    for character in text.chars() {
        if font.characters.contains_key(&(character, params.font_size)) == false {
            font.cache_glyph(character, params.font_size);
        }
        let font_data = &font.characters[&(character, params.font_size)];
        {
            let left_coord = font_data.offset_x as f32 * params.font_scale + total_width;
            let top_coord = params.font_size as f32
                - font_data.glyph_h as f32 * params.font_scale
                - font_data.offset_y as f32 * params.font_scale;

            total_width += font_data.advance * params.font_scale;

            let dest = Rect::new(
                left_coord + x,
                top_coord + y,
                font_data.glyph_w as f32 * params.font_scale,
                font_data.glyph_h as f32 * params.font_scale,
            );

            let source = Rect::new(
                font_data.glyph_x as f32,
                font_data.glyph_y as f32,
                font_data.glyph_w as f32,
                font_data.glyph_h as f32,
            );

            crate::texture::draw_texture_ex(
                font.font_texture,
                dest.x,
                dest.y,
                params.color,
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

pub(crate) struct FontsStorage {
    fonts: Vec<FontInternal>,
}
impl FontsStorage {
    pub(crate) fn new(ctx: &mut miniquad::Context) -> FontsStorage {
        let default_font =
            FontInternal::load_from_bytes(ctx, include_bytes!("../megaui/assets//ProggyClean.ttf"));
        FontsStorage {
            fonts: vec![default_font],
        }
    }

    fn make_font(&mut self, font_internal: FontInternal) -> Font {
        self.fonts.push(font_internal);

        Font(self.fonts.len() - 1)
    }

    fn get_font(&self, font: Font) -> &FontInternal {
        &self.fonts[font.0]
    }

    fn get_font_mut(&mut self, font: Font) -> &mut FontInternal {
        &mut self.fonts[font.0]
    }
}
