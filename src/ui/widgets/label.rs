use crate::{
    math::Vec2,
    ui::{fit, ElementState, Ui, UiContent, UiPosition},
};

use std::borrow::Cow;

pub struct Label<'a> {
    position: UiPosition,
    _multiline: Option<f32>,
    size: Option<Vec2>,
    label: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<S>(label: S) -> Label<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Label {
            position: UiPosition::Auto,
            _multiline: None,
            size: None,
            label: label.into(),
        }
    }

    pub fn multiline(self, line_height: f32) -> Self {
        Label {
            _multiline: Some(line_height),
            ..self
        }
    }

    pub fn position<P: Into<UiPosition>>(self, position: P) -> Self {
        let position = position.into();

        Label { position, ..self }
    }

    pub fn size(self, size: Vec2) -> Self {
        Label {
            size: Some(size),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui) {
        let mut context = ui.get_active_window_context();

        let size = self.size.unwrap_or_else(|| {
            context.window.painter.content_with_margins_size(
                &context.style.label_style,
                &UiContent::Label(self.label.clone()),
            )
        });

        let pos = fit(&mut context, size, self.position);

        context.window.painter.draw_element_content(
            &context.style.label_style,
            pos,
            size,
            &UiContent::Label(self.label),
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
    pub fn label<P: Into<UiPosition>>(&mut self, position: P, label: &str) {
        Label::new(label).position(position).ui(self)
    }

    pub fn calc_size(&mut self, label: &str) -> Vec2 {
        let context = self.get_active_window_context();

        context
            .window
            .painter
            .content_with_margins_size(&context.style.label_style, &UiContent::Label(label.into()))
    }
}
