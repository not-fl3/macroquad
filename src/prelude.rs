//! Mose common types that can be glob-imported `use macroquad::prelude::*` for convenience.

pub use crate::camera::*;
pub use crate::input::*;
pub use crate::material::*;
pub use crate::models::*;
pub use crate::shapes::*;
pub use crate::texture::*;
pub use crate::time::*;
pub use crate::types::*;
pub use crate::ui::*;
pub use crate::window::*;
pub use crate::file::*;

pub use glam;
pub use glam::*;
pub use megaui::hash;
pub use miniquad::{conf::Conf, Comparison, PipelineParams, UniformType};
pub use quad_gl::{colors::*, Color, GlPipeline, QuadGl, Vertex};
pub use quad_rand as rand;

pub use crate::collections;
pub use crate::coroutines;

#[cfg(feature = "log-impl")]
pub use crate::logging::*;
