use crate::{types::Vector2, Layout, Rect, Ui};

use std::borrow::Cow;

pub struct Button<'a> {
    position: Option<Vector2>,
    size: Option<Vector2>,
    label: Cow<'a, str>,
}

impl<'a> Button<'a> {
    pub fn new<S>(label: S) -> Button<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Button {
            position: None,
            size: None,
            label: label.into(),
        }
    }

    pub fn position<P: Into<Option<Vector2>>>(self, position: P) -> Self {
        let position = position.into();

        Button { position, ..self }
    }

    pub fn size(self, size: Vector2) -> Self {
        Button {
            size: Some(size),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui) -> bool {
        let context = ui.get_active_window_context();

        let size = self.size.unwrap_or_else(|| {
            context.window.draw_commands.label_size(&self.label, None)
                + Vector2::new(
                    context.global_style.margin_button * 2.,
                    context.global_style.margin_button,
                )
        });

        let pos = context
            .window
            .cursor
            .fit(size, self.position.map_or(Layout::Vertical, Layout::Free));
        let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
        let hovered = rect.contains(context.input.mouse_position);

        context.window.draw_commands.draw_rect(
            rect,
            None,
            context.global_style.button_background(
                context.focused,
                hovered,
                hovered && context.input.is_mouse_down,
            ),
        );
        context.window.draw_commands.draw_label(
            &self.label,
            pos + Vector2::new(
                context.global_style.margin_button,
                context.global_style.margin_button,
            ),
            Some(context.global_style.text(context.focused)),
        );

        context.focused && hovered && context.input.click_up()
    }
}

impl Ui {
    pub fn button<P: Into<Option<Vector2>>>(&mut self, position: P, label: &str) -> bool {
        Button::new(label).position(position).ui(self)
    }
}
