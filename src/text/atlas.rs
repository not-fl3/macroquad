use crate::{
    math::Rect,
    texture::{Image, Texture2D},
    Color,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub rect: Rect,
}

pub struct Atlas {
    texture: Texture2D,
    image: Image,
    pub sprites: HashMap<u64, Sprite>,
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

    pub fn new(ctx: &mut miniquad::Context, filter: miniquad::FilterMode) -> Atlas {
        let image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let texture = Texture2D {
            texture: miniquad::Texture::from_rgba8(ctx, image.width, image.height, &image.bytes),
        };
        texture
            .raw_miniquad_texture_handle()
            .set_filter(ctx, filter);

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

    pub fn new_unique_id(&mut self) -> u64 {
        self.unique_id += 1;

        self.unique_id
    }

    pub fn get(&self, key: u64) -> Option<Sprite> {
        self.sprites.get(&key).cloned()
    }

    pub fn width(&self) -> u16 {
        self.image.width
    }

    pub fn height(&self) -> u16 {
        self.image.height
    }

    pub fn texture(&mut self) -> Texture2D {
        if self.dirty {
            self.dirty = false;
            if self.texture.width() != self.image.width as _
                || self.texture.height() != self.image.height as _
            {
                self.texture.delete();

                self.texture = Texture2D::from_rgba8(
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                self.texture.set_filter(self.filter);
            }

            self.texture.update(&self.image);
        }

        self.texture
    }

    pub fn get_uv_rect(&self, key: u64) -> Option<Rect> {
        self.get(key).map(|sprite| {
            let w = self.texture.width();
            let h = self.texture.height();

            Rect::new(
                sprite.rect.x / w,
                sprite.rect.y / h,
                sprite.rect.w / w,
                sprite.rect.h / h,
            )
        })
    }

    pub fn cache_sprite(&mut self, key: u64, sprite: Image) {
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
