use crate::{
    math::{Rect, Vec2},
    ui::{ElementState, Layout, Ui},
};

use std::borrow::Cow;

pub struct Button<'a> {
    position: Option<Vec2>,
    size: Option<Vec2>,
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

    pub fn position<P: Into<Option<Vec2>>>(self, position: P) -> Self {
        let position = position.into();

        Button { position, ..self }
    }

    pub fn size(self, size: Vec2) -> Self {
        Button {
            size: Some(size),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui) -> bool {
        let mut context = ui.get_active_window_context();

        let size = self.size.unwrap_or_else(|| {
            context
                .window
                .painter
                .element_size(&context.style.button_style, &self.label)
        });

        let pos = context
            .window
            .cursor
            .fit(size, self.position.map_or(Layout::Vertical, Layout::Free));
        let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
        let (hovered, clicked) = context.register_click_intention(rect);

        context.window.painter.draw_element_background(
            &context.style.button_style,
            pos,
            size,
            ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                selected: false,
            },
        );

        context.window.painter.draw_element_label(
            &context.style.button_style,
            pos,
            &self.label,
            ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                selected: false,
            },
        );

        clicked
    }
}

impl Ui {
    pub fn button<P: Into<Option<Vec2>>>(&mut self, position: P, label: &str) -> bool {
        Button::new(label).position(position).ui(self)
    }
}
