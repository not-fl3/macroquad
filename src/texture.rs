//! Loading and rendering textures. Also render textures, per-pixel image manipluations.

use crate::{get_context, file::load_file, types::Rect};

use glam::{vec2, Vec2};
use quad_gl::{Color, DrawMode, Vertex};

pub use quad_gl::{Image, Texture2D, FilterMode};

/// Load image from file into CPU memory
pub async fn load_image(path: &str) -> Image {
    let bytes = load_file(path)
        .await
        .unwrap_or_else(|e| panic!("Error loading texture: {}", e));

    Image::from_file_with_format(&bytes, None)
}

/// Load texture from file into GPU memory
pub async fn load_texture(path: &str) -> Texture2D {
    let bytes = load_file(path)
        .await
        .unwrap_or_else(|e| panic!("Error loading texture: {}", e));
    let context = &mut get_context().quad_context;

    Texture2D::from_file_with_format(context, &bytes[..], None)
}

pub fn set_texture_filter(texture: Texture2D, filter_mode: FilterMode) {
    let context = &mut get_context().quad_context;

    texture.set_filter(context, filter_mode);
}

/// Upload image data to GPU texture
pub fn update_texture(mut texture: Texture2D, image: &Image) {
    let context = &mut get_context().quad_context;

    texture.update(context, image);
}

pub fn load_texture_from_image(image: &Image) -> Texture2D {
    let context = &mut get_context().quad_context;

    Texture2D::from_rgba8(context, image.width, image.height, &image.bytes)
}

#[derive(Clone, Copy, Debug)]
pub struct RenderTarget {
    pub texture: Texture2D,
    pub render_pass: miniquad::RenderPass,
}

pub fn render_target(width: u32, height: u32) -> RenderTarget {
    let context = &mut get_context().quad_context;

    let texture = miniquad::Texture::new_render_texture(
        context,
        miniquad::TextureParams {
            width,
            height,
            ..Default::default()
        },
    );

    let render_pass = miniquad::RenderPass::new(context, texture, None);

    let texture = Texture2D::from_miniquad_texture(texture);

    RenderTarget {
        texture,
        render_pass,
    }
}

pub struct DrawTextureParams {
    pub dest_size: Option<Vec2>,

    /// Part of texture to draw. If None - draw the whole texture.
    /// Good use example: drawing an image from texture atlas.
    /// Is None by default
    pub source: Option<Rect>,

    /// Rotation in radians
    pub rotation: f32,

    /// Rotation around this point
    pub pivot: Option<Vec2>,
}

impl Default for DrawTextureParams {
    fn default() -> DrawTextureParams {
        DrawTextureParams {
            dest_size: None,
            source: None,
            rotation: 0.,
            pivot: None,
        }
    }
}

pub fn draw_texture(texture: Texture2D, x: f32, y: f32, color: Color) {
    draw_texture_ex(texture, x, y, color, Default::default());
}

pub fn draw_texture_ex(
    texture: Texture2D,
    x: f32,
    y: f32,
    color: Color,
    params: DrawTextureParams,
) {
    let context = &mut get_context().draw_context;

    let Rect {
        x: sx,
        y: sy,
        w: sw,
        h: sh,
    } = params.source.unwrap_or(Rect {
        x: 0.,
        y: 0.,
        w: texture.width(),
        h: texture.height(),
    });

    let (w, h) = params
        .dest_size
        .map_or((texture.width(), texture.height()), |dst| {
            (dst.x(), dst.y())
        });

    let pivot = params.pivot.unwrap_or(vec2(x + w / 2., y + h / 2.));
    let m = pivot;
    let p = [
        vec2(x, y) - pivot,
        vec2(x + w, y) - pivot,
        vec2(x + w, y + h) - pivot,
        vec2(x, y + h) - pivot,
    ];
    let r = params.rotation;
    let p = [
        vec2(
            p[0].x() * r.cos() - p[0].y() * r.sin(),
            p[0].x() * r.sin() + p[0].y() * r.cos(),
        ) + m,
        vec2(
            p[1].x() * r.cos() - p[1].y() * r.sin(),
            p[1].x() * r.sin() + p[1].y() * r.cos(),
        ) + m,
        vec2(
            p[2].x() * r.cos() - p[2].y() * r.sin(),
            p[2].x() * r.sin() + p[2].y() * r.cos(),
        ) + m,
        vec2(
            p[3].x() * r.cos() - p[3].y() * r.sin(),
            p[3].x() * r.sin() + p[3].y() * r.cos(),
        ) + m,
    ];
    #[rustfmt::skip]
    let vertices = [
        Vertex::new(p[0].x(), p[0].y(), 0.,  sx      /texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[1].x(), p[1].y(), 0., (sx + sw)/texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[2].x(), p[2].y(), 0., (sx + sw)/texture.width(), (sy + sh)/texture.height(), color),
        Vertex::new(p[3].x(), p[3].y(), 0.,  sx      /texture.width(), (sy + sh)/texture.height(), color),
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    context.gl.texture(Some(texture));
    context.gl.draw_mode(DrawMode::Triangles);
    context.gl.geometry(&vertices, &indices);
}

#[deprecated(since = "0.3.0", note = "Use draw_texture_ex instead")]
pub fn draw_texture_rec(
    texture: Texture2D,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    color: Color,
) {
    draw_texture_ex(
        texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            source: Some(Rect {
                x: sx,
                y: sy,
                w: sw,
                h: sh,
            }),
            ..Default::default()
        },
    );
}
