use crate::{
    math::Vec2,
    ui::{ElementState, Layout, Ui, UiContent},
};

use std::borrow::Cow;

pub struct Label<'a> {
    position: Option<Vec2>,
    multiline: Option<f32>,
    size: Option<Vec2>,
    label: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<S>(label: S) -> Label<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Label {
            position: None,
            multiline: None,
            size: None,
            label: label.into(),
        }
    }

    pub fn multiline(self, line_height: f32) -> Self {
        Label {
            multiline: Some(line_height),
            ..self
        }
    }

    pub fn position<P: Into<Option<Vec2>>>(self, position: P) -> Self {
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
        let context = ui.get_active_window_context();

        let size = self.size.unwrap_or_else(|| {
            context.window.painter.content_with_margins_size(
                &context.style.label_style,
                &UiContent::Label(self.label.clone()),
            )
        });

        let mut pos = context
            .window
            .cursor
            .fit(size, self.position.map_or(Layout::Vertical, Layout::Free));

        if let Some(line_height) = self.multiline {
            let lines = self.label.lines();
            let last_line_index = lines.clone().count() - 1;
            for (n, line) in lines.enumerate() {
                // need to recaclulate size for each line
                let size = context.window.painter.content_with_margins_size(
                    &context.style.label_style,
                    &UiContent::Label(line.into()),
                );
                context.window.painter.draw_element_content(
                    &context.style.label_style,
                    pos,
                    size,
                    &UiContent::Label(line.into()),
                    ElementState {
                        focused: context.focused,
                        hovered: false,
                        clicked: false,
                        selected: false,
                    },
                );

                pos.y += line_height;
                // only move window cursor if Layout::Vertical and this is not the last line
                if n != last_line_index && self.position.is_none() {
                    context.window.cursor.y += line_height;
                    context.window.cursor.max_row_y = size.y + context.window.cursor.margin;
                }
            }
        } else {
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
}

impl Ui {
    pub fn label<P: Into<Option<Vec2>>>(&mut self, position: P, label: &str) {
        Label::new(label).position(position).ui(self);
    }

    pub fn calc_size(&mut self, label: &str) -> Vec2 {
        let context = self.get_active_window_context();

        context
            .window
            .painter
            .content_with_margins_size(&context.style.label_style, &UiContent::Label(label.into()))
    }
}
