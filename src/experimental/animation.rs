//! Animation management
//!
//! To create custom animation and use it you will need an image that can be represented as tiles.
//! Each tile contains specific frame of animation.
//! Every specific animation should be placed in separate row.
//!
//! # Examples
//! Let's say we have an image of our character where every frame is 15x20 rect and we have this animations:
//! - Idle animation with 20 frames at 12 fps
//! - Run animation with 15 frames at 15 fps
//!
//! ```no_run
//! use macroquad::experimental::animation::*;
//! use macroquad::prelude::*;
//!
//! #[macroquad::main("Animation")]
//! async fn main() {
//!     // Define animations
//!     let mut sprite = AnimatedSprite::new(
//!         15,
//!         20,
//!         &[
//!             Animation {
//!                 name: "idle".to_string(),
//!                 row: 0,
//!                 frames: 20,
//!                 fps: 12,
//!             },
//!             Animation {
//!                 name: "run".to_string(),
//!                 row: 1,
//!                 frames: 15,
//!                 fps: 15,
//!             },
//!         ],
//!         true,
//!     );
//!     let image = load_texture("some_path.png").await.unwrap();
//!     loop {
//!         clear_background(WHITE);
//!         // Now we can draw our character
//!         draw_texture_ex(
//!             image,
//!             10.,
//!             10.,
//!             WHITE,
//!             DrawTextureParams {
//!                 source: Some(sprite.frame().source_rect),
//!                 dest_size: Some(sprite.frame().dest_size),
//!                 ..Default::default()
//!             }
//!         );
//!         // Update frame
//!         sprite.update();
//!         next_frame().await;
//!     }
//! }

use crate::{
    math::{vec2, Rect, Vec2},
    time::get_frame_time,
};

/// Specification of animation
#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

/// Specific animation frame
pub struct AnimationFrame {
    /// Area of current frame in source image
    pub source_rect: Rect,
    /// Size of frame
    pub dest_size: Vec2,
}

/// Main definition of all animations for specific image
#[derive(Clone)]
pub struct AnimatedSprite {
    tile_width: f32,
    tile_height: f32,
    animations: Vec<Animation>,

    current_animation: usize,
    time: f32,
    frame: u32,
    /// Controls if frame should be updated on [update][Self::update]
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

    /// Choose animation to display
    ///
    /// **Note:** the animations is not reset when switching, for this use [set_frame][Self::set_frame]
    pub fn set_animation(&mut self, animation: usize) {
        self.current_animation = animation;

        let animation = &self.animations[self.current_animation];
        self.frame %= animation.frames;
    }

    /// Currently chosen animation
    pub fn current_animation(&self) -> usize {
        self.current_animation
    }

    /// Set specific frame for animation
    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
    }

    /// Update current frame
    ///
    /// Switches to the next frame every `1. / current_animation.fps` seconds
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

    /// Get current frame
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
