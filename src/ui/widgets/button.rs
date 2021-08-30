use crate::{
    math::{Rect, Vec2},
    ui::{ElementState, Layout, Ui, UiContent},
};

pub struct Button<'a> {
    position: Option<Vec2>,
    size: Option<Vec2>,
    content: UiContent<'a>,
}

impl<'a> Button<'a> {
    pub fn new<S>(content: S) -> Button<'a>
    where
        S: Into<UiContent<'a>>,
    {
        Button {
            position: None,
            size: None,
            content: content.into(),
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
                .content_with_margins_size(&context.style.button_style, &self.content)
        });

        let pos = context
            .window
            .cursor
            .fit(size, self.position.map_or(Layout::Vertical, Layout::Free));
        let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
        let (hovered, clicked) = context.register_click_intention(rect);

        if !context.style.button_style.reverse_background_z {
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
        }

        // this is not entirely correct
        // there should be a distinction of button background style
        // button label style and button content style
        // waiting for Style refactoring, and so far - using special Label's style
        // for button text color
        let style = match self.content {
            UiContent::Label(..) => &context.style.button_text_style,
            UiContent::Texture(..) => &context.style.button_style,
        };
        context.window.painter.draw_element_content(
            style,
            pos,
            size,
            &self.content,
            ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                selected: false,
            },
        );

        if context.style.button_style.reverse_background_z {
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
        }

        clicked
    }
}

impl Ui {
    pub fn button<'a, P: Into<Option<Vec2>>, S: Into<UiContent<'a>>>(
        &mut self,
        position: P,
        label: S,
    ) -> bool {
        Button::new(label).position(position).ui(self)
    }
}
