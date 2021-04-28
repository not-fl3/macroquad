//! Loading and rendering textures. Also render textures, per-pixel image manipulations.

use crate::{
    file::{load_file, FileError},
    get_context,
    math::Rect,
};

use crate::quad_gl::{Color, DrawMode, Vertex};
use glam::{vec2, Vec2};

pub use crate::quad_gl::FilterMode;

/// Image, data stored in CPU memory
#[derive(Clone)]
pub struct Image {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes.len()", &self.bytes.len())
            .finish()
    }
}

impl Image {
    /// Creates an empty Image.
    ///
    /// ```
    /// # use macroquad::prelude::*;
    /// let image = Image::empty();
    /// ```
    pub fn empty() -> Image {
        Image {
            width: 0,
            height: 0,
            bytes: vec![],
        }
    }

    /// Creates an Image from a slice of bytes that contains an encoded image.
    ///
    /// If `format` is None, it will make an educated guess on the
    /// [ImageFormat][image::ImageFormat].
    ///
    /// # Example
    ///
    /// ```
    /// # use macroquad::prelude::*;
    /// let icon = Image::from_file_with_format(
    ///     include_bytes!("../examples/rust.png"),
    ///     Some(ImageFormat::Png),
    ///     );
    /// ```
    pub fn from_file_with_format(bytes: &[u8], format: Option<image::ImageFormat>) -> Image {
        let img = if let Some(fmt) = format {
            image::load_from_memory_with_format(&bytes, fmt)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba8()
        } else {
            image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba8()
        };
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Image {
            width,
            height,
            bytes,
        }
    }

    /// Creates an Image filled with the provided [Color].
    pub fn gen_image_color(width: u16, height: u16, color: Color) -> Image {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        for i in 0..width as usize * height as usize {
            bytes[i * 4 + 0] = (color.r * 255.) as u8;
            bytes[i * 4 + 1] = (color.g * 255.) as u8;
            bytes[i * 4 + 2] = (color.b * 255.) as u8;
            bytes[i * 4 + 3] = (color.a * 255.) as u8;
        }
        Image {
            width,
            height,
            bytes,
        }
    }

    /// Updates this image from a slice of [Color]s.
    pub fn update(&mut self, colors: &[Color]) {
        assert!(self.width as usize * self.height as usize == colors.len());

        for i in 0..colors.len() {
            self.bytes[i * 4] = (colors[i].r * 255.) as u8;
            self.bytes[i * 4 + 1] = (colors[i].g * 255.) as u8;
            self.bytes[i * 4 + 2] = (colors[i].b * 255.) as u8;
            self.bytes[i * 4 + 3] = (colors[i].a * 255.) as u8;
        }
    }

    /// Returns the width of this image.
    pub fn width(&self) -> usize {
        self.width as usize
    }

    /// Returns the height of this image.
    pub fn height(&self) -> usize {
        self.height as usize
    }

    /// Returns this image's data as a slice of 4-byte arrays.
    pub fn get_image_data(&self) -> &[[u8; 4]] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                self.bytes.as_ptr() as *const [u8; 4],
                self.width as usize * self.height as usize,
            )
        }
    }

    /// Returns this image's data as a mutable slice of 4-byte arrays.
    pub fn get_image_data_mut(&mut self) -> &mut [[u8; 4]] {
        use std::slice;

        unsafe {
            slice::from_raw_parts_mut(
                self.bytes.as_mut_ptr() as *mut [u8; 4],
                self.width as usize * self.height as usize,
            )
        }
    }

    /// Modifies a pixel [Color] in this image.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let width = self.width;

        self.get_image_data_mut()[(y * width as u32 + x) as usize] = color.into();
    }

    /// Returns a pixel [Color] from this image.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        self.get_image_data()[(y * self.width as u32 + x) as usize].into()
    }

    /// Returns an Image from a rect inside this image.
    pub fn sub_image(&self, rect: Rect) -> Image {
        let width = rect.w as usize;
        let height = rect.h as usize;
        let mut bytes = vec![0; width * height * 4];

        let x = rect.x as usize;
        let y = rect.y as usize;
        let mut n = 0;
        for y in y..y + height {
            for x in x..x + width {
                bytes[n] = self.bytes[y * self.width as usize * 4 + x * 4 + 0];
                bytes[n + 1] = self.bytes[y * self.width as usize * 4 + x * 4 + 1];
                bytes[n + 2] = self.bytes[y * self.width as usize * 4 + x * 4 + 2];
                bytes[n + 3] = self.bytes[y * self.width as usize * 4 + x * 4 + 3];
                n += 4;
            }
        }
        Image {
            width: width as u16,
            height: height as u16,
            bytes,
        }
    }

    /// Saves this image as a PNG file.
    pub fn export_png(&self, path: &str) {
        let mut bytes = vec![0; self.width as usize * self.height as usize * 4];

        // flip the image before saving
        for y in 0..self.height as usize {
            for x in 0..self.width as usize * 4 {
                bytes[y * self.width as usize * 4 + x] =
                    self.bytes[(self.height as usize - y - 1) * self.width as usize * 4 + x];
            }
        }

        image::save_buffer(
            path,
            &bytes[..],
            self.width as _,
            self.height as _,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
}

/// Loads an [Image] from a file into CPU memory.
pub async fn load_image(path: &str) -> Result<Image, FileError> {
    let bytes = load_file(path).await?;

    Ok(Image::from_file_with_format(&bytes, None))
}

/// Loads a [Texture2D] from a file into GPU memory.
pub async fn load_texture(path: &str) -> Result<Texture2D, FileError> {
    let bytes = load_file(path).await?;

    Ok(Texture2D::from_file_with_format(&bytes[..], None))
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

#[derive(Debug, Clone)]
pub struct DrawTextureParams {
    pub dest_size: Option<Vec2>,

    /// Part of texture to draw. If None - draw the whole texture.
    /// Good use example: drawing an image from texture atlas.
    /// Is None by default
    pub source: Option<Rect>,

    /// Rotation in radians
    pub rotation: f32,

    /// Mirror on the X axis
    pub flip_x: bool,

    /// Mirror on the Y axis
    pub flip_y: bool,

    /// Rotate around this point.
    /// When `None`, rotate around the texture's center.
    /// When `Some`, the coordinates are in screen-space.
    /// E.g. pivot (0,0) rotates around the top left corner of the screen, not of the
    pub pivot: Option<Vec2>,
}

impl Default for DrawTextureParams {
    fn default() -> DrawTextureParams {
        DrawTextureParams {
            dest_size: None,
            source: None,
            rotation: 0.,
            pivot: None,
            flip_x: false,
            flip_y: false,
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

    let (mut w, mut h) = match params.dest_size {
        Some(dst) => (dst.x, dst.y),
        _ => (sw, sh),
    };
    let mut x = x;
    let mut y = y;
    if params.flip_x {
        x = x + w;
        w = -w;
    }
    if params.flip_y {
        y = y + h;
        h = -h;
    }

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
            p[0].x * r.cos() - p[0].y * r.sin(),
            p[0].x * r.sin() + p[0].y * r.cos(),
        ) + m,
        vec2(
            p[1].x * r.cos() - p[1].y * r.sin(),
            p[1].x * r.sin() + p[1].y * r.cos(),
        ) + m,
        vec2(
            p[2].x * r.cos() - p[2].y * r.sin(),
            p[2].x * r.sin() + p[2].y * r.cos(),
        ) + m,
        vec2(
            p[3].x * r.cos() - p[3].y * r.sin(),
            p[3].x * r.sin() + p[3].y * r.cos(),
        ) + m,
    ];
    #[rustfmt::skip]
    let vertices = [
        Vertex::new(p[0].x, p[0].y, 0.,  sx      /texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[1].x, p[1].y, 0., (sx + sw)/texture.width(),  sy      /texture.height(), color),
        Vertex::new(p[2].x, p[2].y, 0., (sx + sw)/texture.width(), (sy + sh)/texture.height(), color),
        Vertex::new(p[3].x, p[3].y, 0.,  sx      /texture.width(), (sy + sh)/texture.height(), color),
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

/// Get pixel data from screen buffer and return an Image (screenshot)
pub fn get_screen_data() -> Image {
    unsafe {
        crate::window::get_internal_gl().flush();
    }

    let context = get_context();

    let texture = Texture2D::from_miniquad_texture(miniquad::Texture::new_render_texture(
        &mut context.quad_context,
        miniquad::TextureParams {
            width: context.screen_width as _,
            height: context.screen_height as _,
            ..Default::default()
        },
    ));

    texture.grab_screen();

    texture.get_texture_data()
}

/// Texture, data stored in GPU memory
#[derive(Clone, Copy, Debug)]
pub struct Texture2D {
    pub(crate) texture: miniquad::Texture,
}

impl Texture2D {
    /// Creates an empty Texture2D.
    ///
    /// # Example
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// let texture = Texture2D::empty();
    /// # }
    /// ```
    pub fn empty() -> Texture2D {
        Texture2D {
            texture: miniquad::Texture::empty(),
        }
    }

    /// Creates a Texture2D from a slice of bytes that contains an encoded image.
    ///
    /// If `format` is None, it will make an educated guess on the
    /// [ImageFormat][image::ImageFormat].
    ///
    /// # Example
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// let texture = Texture2D::from_file_with_format(
    ///     include_bytes!("../examples/rust.png"),
    ///     None,
    ///     );
    /// # }
    /// ```
    pub fn from_file_with_format<'a>(
        bytes: &[u8],
        format: Option<image::ImageFormat>,
    ) -> Texture2D {
        let img = if let Some(fmt) = format {
            image::load_from_memory_with_format(&bytes, fmt)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba8()
        } else {
            image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("{}", e))
                .to_rgba8()
        };
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Self::from_rgba8(width, height, &bytes)
    }

    /// Creates a Texture2D from an [Image].
    pub fn from_image(image: &Image) -> Texture2D {
        Texture2D::from_rgba8(image.width, image.height, &image.bytes)
    }

    /// Creates a Texture2D from a miniquad
    /// [Texture](https://docs.rs/miniquad/0.3.0-alpha/miniquad/graphics/struct.Texture.html)
    pub fn from_miniquad_texture(texture: miniquad::Texture) -> Texture2D {
        Texture2D { texture }
    }

    /// Creates a Texture2D from a slice of bytes in an R,G,B,A sequence,
    /// with the given width and height.
    ///
    /// # Example
    ///
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// // Create a 2x2 texture from a byte slice with 4 rgba pixels
    /// let bytes: Vec<u8> = vec![255, 0, 0, 192, 0, 255, 0, 192, 0, 0, 255, 192, 255, 255, 255, 192];
    /// let texture = Texture2D::from_rgba8(2, 2, &bytes);
    /// # }
    /// ```
    pub fn from_rgba8(width: u16, height: u16, bytes: &[u8]) -> Texture2D {
        let ctx = &mut get_context().quad_context;

        Texture2D {
            texture: miniquad::Texture::from_rgba8(ctx, width, height, bytes),
        }
    }

    /// Uploads [Image] data to this texture.
    pub fn update(&self, image: &Image) {
        assert_eq!(self.texture.width, image.width as u32);
        assert_eq!(self.texture.height, image.height as u32);

        let ctx = &mut get_context().quad_context;

        self.texture.update(ctx, &image.bytes);
    }

    /// Returns the width of this texture.
    pub fn width(&self) -> f32 {
        self.texture.width as f32
    }

    /// Returns the height of this texture.
    pub fn height(&self) -> f32 {
        self.texture.height as f32
    }

    /// Sets the [FilterMode] of this texture.
    ///
    /// Use Nearest if you need integer-ratio scaling for pixel art, for example.
    ///
    /// # Example
    /// ```
    /// # use macroquad::prelude::*;
    /// # #[macroquad::main("test")]
    /// # async fn main() {
    /// let texture = Texture2D::empty();
    /// texture.set_filter(FilterMode::Linear);
    /// # }
    /// ```
    pub fn set_filter(&self, filter_mode: FilterMode) {
        let ctx = &mut get_context().quad_context;

        self.texture.set_filter(ctx, filter_mode);
    }

    /// Returns the handle for this texture.
    pub fn raw_miniquad_texture_handle(&self) -> miniquad::Texture {
        self.texture
    }

    /// Updates this texture from the screen.
    pub fn grab_screen(&self) {
        use miniquad::*;

        let (internal_format, _, _) = self.texture.format.into();
        unsafe {
            gl::glBindTexture(gl::GL_TEXTURE_2D, self.texture.gl_internal_id());
            gl::glCopyTexImage2D(
                gl::GL_TEXTURE_2D,
                0,
                internal_format,
                0,
                0,
                self.texture.width as _,
                self.texture.height as _,
                0,
            );
        }
    }

    /// Returns an [Image] from the pixel data in this texture.
    ///
    /// This operation can be expensive.
    pub fn get_texture_data(&self) -> Image {
        let mut image = Image {
            width: self.texture.width as _,
            height: self.texture.height as _,
            bytes: vec![0; self.texture.width as usize * self.texture.height as usize * 4],
        };

        self.texture.read_pixels(&mut image.bytes);

        image
    }

    /// Unloads texture from GPU memory.
    ///
    /// Using a deleted texture could give different results on different
    /// platforms and is not recommended.
    pub fn delete(&self) {
        self.raw_miniquad_texture_handle().delete()
    }
}
