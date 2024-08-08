use crate::{
    math::{Rect, Vec2},
    texture::Texture2D,
    ui::{fit, Ui, UiPosition},
};

pub struct Texture {
    position: UiPosition,
    w: f32,
    h: f32,
    texture: Texture2D,
}

impl Texture {
    pub fn new(texture: Texture2D) -> Texture {
        Texture {
            position: UiPosition::Auto,
            w: 100.,
            h: 100.,
            texture,
        }
    }

    pub fn size(self, w: f32, h: f32) -> Self {
        Texture { w, h, ..self }
    }

    pub fn position<P: Into<UiPosition>>(self, position: P) -> Self {
        let position = position.into();

        Texture { position, ..self }
    }

    pub fn ui(self, ui: &mut Ui) -> bool {
        let mut context = ui.get_active_window_context();

        let size = Vec2::new(self.w, self.h);

        let pos = fit(&mut context, size, self.position);
        context
            .window
            .painter
            .draw_raw_texture(Rect::new(pos.x, pos.y, self.w, self.h), &self.texture);

        let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
        let hovered = rect.contains(context.input.mouse_position);

        context.focused && hovered && context.input.click_up
    }
}

impl Ui {
    pub fn texture(&mut self, texture: Texture2D, w: f32, h: f32) -> bool {
        Texture::new(texture).size(w, h).ui(self)
    }
}
