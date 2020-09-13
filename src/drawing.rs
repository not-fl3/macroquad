//! this is legacy and going to disappear soon

use quad_gl::QuadGl;

pub use quad_gl::{colors::*, Color, DrawMode, FilterMode, Image, Texture2D};

use glam::Mat4;

pub struct DrawContext {
    pub(crate) font_texture: Texture2D,
    pub(crate) gl: QuadGl,
    pub(crate) camera_matrix: Option<Mat4>,
    pub(crate) current_pass: Option<miniquad::RenderPass>,
    pub ui: megaui::Ui,
    ui_draw_list: Vec<megaui::DrawList>,
}

impl DrawContext {
    pub fn new(ctx: &mut miniquad::Context) -> DrawContext {
        let mut ui = megaui::Ui::new();
        ui.set_clipboard_object(crate::ui::ClipboardObject);

        let texture_data = &ui.font_atlas.texture;
        let font_texture = Texture2D::from_rgba8(
            ctx,
            texture_data.width as u16,
            texture_data.height as u16,
            &texture_data.data,
        );
        let mut draw_context = DrawContext {
            camera_matrix: None,
            gl: QuadGl::new(ctx),
            font_texture,
            ui,
            ui_draw_list: Vec::with_capacity(10000),
            current_pass: None,
        };

        draw_context.update_projection_matrix(ctx);

        draw_context
    }

    fn draw_ui(&mut self, _: &mut miniquad::Context) {
        self.ui_draw_list.clear();

        self.ui.render(&mut self.ui_draw_list);
        self.ui.new_frame();

        let mut ui_draw_list = vec![];

        std::mem::swap(&mut ui_draw_list, &mut self.ui_draw_list);
        self.gl.texture(Some(self.font_texture));

        for draw_command in &ui_draw_list {
            self.gl.scissor(
                draw_command
                    .clipping_zone
                    .map(|rect| (rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32)),
            );
            self.gl.draw_mode(DrawMode::Triangles);
            self.gl
                .geometry(&draw_command.vertices, &draw_command.indices);
        }
        self.gl.texture(None);

        std::mem::swap(&mut ui_draw_list, &mut self.ui_draw_list);
    }

    pub(crate) fn perform_render_passes(&mut self, ctx: &mut miniquad::Context) {
        self.draw_ui(ctx);
        self.gl.draw(ctx);
    }

    pub(crate) fn update_projection_matrix(&mut self, ctx: &mut miniquad::Context) {
        let (width, height) = ctx.screen_size();

        let mut projection = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);

        if let Some(matrix) = self.camera_matrix {
            projection = matrix;
        }

        self.gl.set_projection_matrix(projection);
    }
}
