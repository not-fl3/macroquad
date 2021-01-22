use crate::{
    color,
    math::{vec2, Rect, Vec2},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
    time::get_frame_time,
};

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

pub struct AnimatedSprite {
    texture: Texture2D,
    tile_width: f32,
    tile_height: f32,
    animations: Vec<Animation>,

    current_animation: usize,
    time: f32,
    frame: u32,
}

impl AnimatedSprite {
    pub fn new(texture: (Texture2D, u32, u32), animations: &[Animation]) -> AnimatedSprite {
        AnimatedSprite {
            texture: texture.0,
            tile_width: texture.1 as f32,
            tile_height: texture.2 as f32,
            animations: animations.to_vec(),
            current_animation: 0,
            time: 0.0,
            frame: 0,
        }
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.current_animation = animation;
    }

    pub fn draw(&mut self, pos: Vec2, flip_x: bool, _flip_y: bool) {
        let animation = &self.animations[self.current_animation];

        let x_sign = if flip_x { 1. } else { -1. };
        self.frame %= animation.frames;
        draw_texture_ex(
            self.texture,
            pos.x + self.tile_width * !flip_x as i32 as f32,
            pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    self.tile_width * self.frame as f32,
                    self.tile_height * animation.row as f32,
                    self.tile_width,
                    self.tile_height,
                )),
                dest_size: Some(vec2(x_sign * self.tile_width, self.tile_height)),
                ..Default::default()
            },
        );
        self.time += get_frame_time();
        if self.time > 1. / animation.fps as f32 {
            self.frame += 1;
            self.time = 0.0;
        }
    }
}
