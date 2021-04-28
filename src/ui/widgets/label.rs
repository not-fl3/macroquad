use crate::{
    math::Vec2,
    ui::{ElementState, Layout, Ui},
};

use std::borrow::Cow;

pub struct Label<'a> {
    position: Option<Vec2>,
    _multiline: Option<f32>,
    label: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<S>(label: S) -> Label<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Label {
            position: None,
            _multiline: None,
            label: label.into(),
        }
    }

    pub fn multiline(self, line_height: f32) -> Self {
        Label {
            _multiline: Some(line_height),
            ..self
        }
    }

    pub fn position<P: Into<Option<Vec2>>>(self, position: P) -> Self {
        let position = position.into();

        Label { position, ..self }
    }

    pub fn ui(self, ui: &mut Ui) {
        let context = ui.get_active_window_context();

        let size = context
            .window
            .painter
            .element_size(&context.style.label_style, &self.label);

        let pos = context
            .window
            .cursor
            .fit(size, self.position.map_or(Layout::Vertical, Layout::Free));

        context.window.painter.draw_element_label(
            &context.style.label_style,
            pos,
            &self.label,
            ElementState {
                focused: context.focused,
                hovered: false,
                clicked: false,
                selected: false,
            },
        );
    }
}

impl Ui {
    pub fn label<P: Into<Option<Vec2>>>(&mut self, position: P, label: &str) {
        Label::new(label).position(position).ui(self)
    }

    pub fn calc_size(&mut self, label: &str) -> Vec2 {
        let context = self.get_active_window_context();

        context
            .window
            .painter
            .element_size(&context.style.label_style, label)
    }
}
