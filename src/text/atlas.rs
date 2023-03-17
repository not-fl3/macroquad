use crate::{
    get_quad_context,
    math::Rect,
    texture::{Image, Texture2D},
    Color,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub rect: Rect,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SpriteKey {
    Texture(miniquad::TextureId),
    Id(u64),
}
pub struct Atlas {
    texture: miniquad::TextureId,
    image: Image,
    pub sprites: HashMap<SpriteKey, Sprite>,
    cursor_x: u16,
    cursor_y: u16,
    max_line_height: u16,

    pub dirty: bool,

    filter: miniquad::FilterMode,

    unique_id: u64,
}

impl Atlas {
    // pixel gap between glyphs in the atlas
    const GAP: u16 = 2;
    // well..
    const UNIQUENESS_OFFSET: u64 = 100000;

    pub fn new(ctx: &mut dyn miniquad::RenderingBackend, filter: miniquad::FilterMode) -> Atlas {
        let image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let texture = ctx.new_texture_from_rgba8(image.width, image.height, &image.bytes);
        ctx.texture_set_filter(texture, miniquad::FilterMode::Nearest);

        Atlas {
            image,
            texture,
            cursor_x: 0,
            cursor_y: 0,
            dirty: false,
            max_line_height: 0,
            sprites: HashMap::new(),
            filter,
            unique_id: Self::UNIQUENESS_OFFSET,
        }
    }

    pub fn new_unique_id(&mut self) -> SpriteKey {
        self.unique_id += 1;

        SpriteKey::Id(self.unique_id)
    }

    pub fn set_filter(&mut self, filter_mode: miniquad::FilterMode) {
        let ctx = get_quad_context();
        self.filter = filter_mode;
        ctx.texture_set_filter(self.texture, filter_mode);
    }

    pub fn get(&self, key: SpriteKey) -> Option<Sprite> {
        self.sprites.get(&key).cloned()
    }

    pub fn width(&self) -> u16 {
        self.image.width
    }

    pub fn height(&self) -> u16 {
        self.image.height
    }

    pub fn texture(&mut self) -> miniquad::TextureId {
        let ctx = get_quad_context();
        if self.dirty {
            self.dirty = false;
            let (texture_width, texture_height) = ctx.texture_size(self.texture);
            if texture_width != self.image.width as _ || texture_height != self.image.height as _ {
                ctx.delete_texture(self.texture);

                self.texture = ctx.new_texture_from_rgba8(
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                ctx.texture_set_filter(self.texture, self.filter);
            }

            ctx.texture_update(self.texture, &self.image.bytes);
        }

        self.texture
    }

    pub fn get_uv_rect(&self, key: SpriteKey) -> Option<Rect> {
        let ctx = get_quad_context();
        self.get(key).map(|sprite| {
            let (w, h) = ctx.texture_size(self.texture);

            Rect::new(
                sprite.rect.x / w as f32,
                sprite.rect.y / h as f32,
                sprite.rect.w / w as f32,
                sprite.rect.h / h as f32,
            )
        })
    }

    pub fn cache_sprite(&mut self, key: SpriteKey, sprite: Image) {
        let (width, height) = (sprite.width as usize, sprite.height as usize);

        let x = if self.cursor_x + (width as u16) < self.image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x + Self::GAP;
            self.cursor_x += width as u16 + Self::GAP * 2;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP * 2;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };
        let y = self.cursor_y;

        // texture bounds exceeded
        if self.cursor_y + height as u16 > self.image.height {
            // reset glyph cache state
            let sprites = self.sprites.drain().collect::<Vec<_>>();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.max_line_height = 0;

            let old_image = self.image.clone();

            // increase font texture size
            self.image = Image::gen_image_color(
                self.image.width * 2,
                self.image.height * 2,
                Color::new(0.0, 0.0, 0.0, 0.0),
            );

            // recache all previously cached symbols
            for (key, sprite) in sprites {
                let image = old_image.sub_image(sprite.rect);
                self.cache_sprite(key, image);
            }

            // cache the new sprite
            self.cache_sprite(key, sprite);
        } else {
            self.dirty = true;

            for j in 0..height {
                for i in 0..width {
                    self.image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        sprite.get_pixel(i as u32, j as u32),
                    );
                }
            }

            self.sprites.insert(
                key,
                Sprite {
                    rect: Rect::new(x as f32, y as f32, width as f32, height as f32),
                },
            );
        }
    }
}
