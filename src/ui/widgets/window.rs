use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, Ui, UiContent, WindowContext},
};

#[derive(Debug, Clone)]
pub struct Window {
    id: Id,
    position: Vec2,
    size: Vec2,
    close_button: bool,
    movable: bool,
    titlebar: bool,
    label: Option<String>,
}

impl Window {
    pub fn new(id: Id, position: Vec2, size: Vec2) -> Window {
        Window {
            id,
            position,
            size,
            close_button: false,
            movable: true,
            titlebar: true,
            label: None,
        }
    }

    pub fn label(self, label: &str) -> Window {
        Window {
            label: Some(label.to_string()),
            ..self
        }
    }

    /// If moveable is set to true then it means that the user
    /// can drag the window (default).
    ///
    /// After the first frame the window got drawn it will stop looking at the position given
    /// to it and instead will have its position be fully in control by the user.
    ///
    ///
    /// If on the other hand it is false then the position of this window will
    /// always be equal to the value given.
    pub fn movable(self, movable: bool) -> Window {
        Window { movable, ..self }
    }

    pub fn close_button(self, close_button: bool) -> Window {
        Window {
            close_button,
            ..self
        }
    }

    pub fn titlebar(self, titlebar: bool) -> Window {
        Window { titlebar, ..self }
    }

    pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) -> bool {
        let token = self.begin(ui);
        f(ui);
        token.end(ui)
    }

    pub fn begin(self, ui: &mut Ui) -> WindowToken {
        let context = ui.begin_window(
            self.id,
            None,
            self.position,
            self.size,
            self.titlebar,
            self.movable,
        );

        // TODO: this will make each new window focused(appeared on the top) always
        // consider adding some configuration to be able to spawn background windows
        if context.window.was_active == false {
            ui.focus_window(self.id);
        }

        let mut context = ui.get_active_window_context();

        self.draw_window_frame(&mut context);
        if self.close_button && self.draw_close_button(&mut context) {
            context.close();
        }

        let clip_rect = context.window.content_rect();
        context.scroll_area();

        context.window.painter.clip(clip_rect);

        WindowToken
    }

    fn draw_close_button(&self, context: &mut WindowContext) -> bool {
        let style = context.style;
        let size = Vec2::new(style.title_height - 4., style.title_height - 4.);
        let pos = Vec2::new(
            context.window.position.x + context.window.size.x - style.title_height + 1.,
            context.window.position.y + 2.,
        );
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

        clicked
    }

    fn draw_window_frame(&self, context: &mut WindowContext) {
        let focused = context.focused;
        let style = context.style;
        let position = context.window.position;
        let size = context.window.size;

        context.window.painter.draw_element_background(
            &style.window_style,
            position,
            size,
            ElementState {
                focused,
                hovered: false,
                clicked: false,
                selected: false,
            },
        );

        // TODO: figure what does title bar mean with windows with background
        if self.titlebar {
            if let Some(label) = &self.label {
                context.window.painter.draw_element_content(
                    &context.style.window_titlebar_style,
                    position,
                    vec2(size.x, style.title_height),
                    &UiContent::Label(label.into()),
                    ElementState {
                        focused,
                        clicked: false,
                        hovered: false,
                        selected: false,
                    },
                );
            }
            context.window.painter.draw_line(
                vec2(position.x, position.y + style.title_height),
                vec2(position.x + size.x, position.y + style.title_height),
                style.window_titlebar_style.color(ElementState {
                    focused,
                    clicked: false,
                    hovered: false,
                    selected: false,
                }),
            );
        }
    }
}

#[must_use = "Must call `.end()` to finish Window"]
pub struct WindowToken;

impl WindowToken {
    pub fn end(self, ui: &mut Ui) -> bool {
        let context = ui.get_active_window_context();
        context.window.painter.clip(None);

        let opened = context.window.want_close == false;

        ui.end_window();

        opened
    }
}

impl Ui {
    pub fn window<F: FnOnce(&mut Ui)>(&mut self, id: Id, position: Vec2, size: Vec2, f: F) -> bool {
        Window::new(id, position, size).titlebar(false).ui(self, f)
    }
}
