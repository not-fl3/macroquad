//! this is legacy and going to disappear soon

use quad_gl::{QuadGl, Vertex};

pub use quad_gl::{colors::*, Color, DrawMode, FilterMode, Image, Texture2D};

use glam::Mat4;

#[cfg(feature="megaui")]
pub struct UiDrawContext {
    ui_draw_list: Vec<megaui::DrawList>,
    pub(crate) ui: megaui::Ui,
    pub(crate) font_texture: Texture2D,
}

#[cfg(feature="megaui")]
impl UiDrawContext {
    fn new(ctx: &mut miniquad::Context) -> Self {
        let mut ui = megaui::Ui::new();
        ui.set_clipboard_object(crate::ui::ClipboardObject);

        let texture_data = &ui.font_atlas.texture;
        let font_texture = Texture2D::from_rgba8(
            ctx,
            texture_data.width as u16,
            texture_data.height as u16,
            &texture_data.data,
        );

        Self {
            font_texture,
            ui,
            ui_draw_list: Vec::with_capacity(10_000),
        }
    }

    fn draw(&mut self, gl: &mut QuadGl, _: &mut miniquad::Context) {
        self.ui_draw_list.clear();

        self.ui.render(&mut self.ui_draw_list);
        self.ui.new_frame();

        gl.texture(Some(self.font_texture));

        for draw_command in self.ui_draw_list.drain(..) {
            gl.scissor(
                draw_command
                    .clipping_zone
                    .map(|rect| (rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32)),
            );
            gl.draw_mode(DrawMode::Triangles);
            gl
                .geometry(&draw_command.vertices, &draw_command.indices);
        }
        gl.texture(None);
    }
}

pub struct DrawContext {
    pub(crate) gl: QuadGl,
    pub(crate) camera_matrix: Option<Mat4>,
    pub(crate) current_pass: Option<miniquad::RenderPass>,
    #[cfg(feature="megaui")]
    pub(crate) ui_ctx: UiDrawContext,
}

impl DrawContext {
    pub fn new(ctx: &mut miniquad::Context) -> DrawContext {
        let mut draw_context = DrawContext {
            gl: QuadGl::new(ctx),
            camera_matrix: None,
            current_pass: None,
            #[cfg(feature="megaui")]
            ui_ctx: UiDrawContext::new(ctx),
        };

        draw_context.update_projection_matrix(ctx);

        draw_context
    }

    #[cfg(feature="megaui")]
    pub(crate) fn ui(&self) -> &megaui::Ui {
        &self.ui_ctx.ui
    }

    #[cfg(feature="megaui")]
    pub(crate) fn ui_mut(&mut self) -> &mut megaui::Ui {
        &mut self.ui_ctx.ui
    }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        #[rustfmt::skip]
        let vertices = [
            Vertex::new(x    , y    , 0., 0.0, 1.0, color),
            Vertex::new(x + w, y    , 0., 1.0, 0.0, color),
            Vertex::new(x + w, y + h, 0., 1.0, 1.0, color),
            Vertex::new(x    , y + h, 0., 0.0, 0.0, color),
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        self.gl.texture(None);
        self.gl.draw_mode(DrawMode::Triangles);
        self.gl.geometry(&vertices, &indices);
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let nx = -dy; // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment
        let ny = dx;

        let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
        if tlen < std::f32::EPSILON {
            return;
        }
        let tx = nx / tlen;
        let ty = ny / tlen;

        self.gl.texture(None);
        self.gl.draw_mode(DrawMode::Triangles);
        self.gl.geometry(
            &[
                Vertex::new(x1 + tx, y1 + ty, 0., 0., 0., color),
                Vertex::new(x1 - tx, y1 - ty, 0., 0., 0., color),
                Vertex::new(x2 + tx, y2 + ty, 0., 0., 0., color),
                Vertex::new(x2 - tx, y2 - ty, 0., 0., 0., color),
            ],
            &[0, 1, 2, 2, 1, 3],
        );
    }

    pub(crate) fn perform_render_passes(&mut self, ctx: &mut miniquad::Context) {
        #[cfg(feature="megaui")]
        self.ui_ctx.draw(&mut self.gl, ctx);
        self.gl.draw(ctx);
    }

    pub fn update_projection_matrix(&mut self, ctx: &mut miniquad::Context) {
        let (width, height) = ctx.screen_size();

        let mut projection = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);

        if let Some(matrix) = self.camera_matrix {
            projection = matrix;
        }

        self.gl.set_projection_matrix(projection);
    }
}
