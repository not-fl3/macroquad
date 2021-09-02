use crate::{
    math::{vec2, Rect, Vec2},
    time::get_frame_time,
};

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

pub struct AnimationFrame {
    pub source_rect: Rect,
    pub dest_size: Vec2,
}

#[derive(Clone)]
pub struct AnimatedSprite {
    tile_width: f32,
    tile_height: f32,
    animations: Vec<Animation>,

    current_animation: usize,
    time: f32,
    frame: u32,
    pub playing: bool,
}

impl AnimatedSprite {
    pub fn new(
        tile_width: u32,
        tile_height: u32,
        animations: &[Animation],
        playing: bool,
    ) -> AnimatedSprite {
        AnimatedSprite {
            tile_width: tile_width as f32,
            tile_height: tile_height as f32,
            animations: animations.to_vec(),
            current_animation: 0,
            time: 0.0,
            frame: 0,
            playing,
        }
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.current_animation = animation;

        let animation = &self.animations[self.current_animation];
        self.frame %= animation.frames;
    }

    pub fn current_animation(&self) -> usize {
        self.current_animation
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
    }

    pub fn update(&mut self) {
        let animation = &self.animations[self.current_animation];

        if self.playing {
            self.time += get_frame_time();
            if self.time > 1. / animation.fps as f32 {
                self.frame += 1;
                self.time = 0.0;
            }
        }
        self.frame %= animation.frames;
    }

    pub fn frame(&self) -> AnimationFrame {
        let animation = &self.animations[self.current_animation];

        AnimationFrame {
            source_rect: Rect::new(
                self.tile_width * self.frame as f32,
                self.tile_height * animation.row as f32,
                self.tile_width,
                self.tile_height,
            ),
            dest_size: vec2(self.tile_width, self.tile_height),
        }
    }
}
