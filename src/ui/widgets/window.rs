use crate::{
    math::Vec2,
    ui::{ElementState, Id, Ui, WindowContext},
};

#[derive(Debug, Clone)]
pub struct Window {
    id: Id,
    position: Vec2,
    size: Vec2,
    close_button: bool,
    enabled: bool,
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
            enabled: true,
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

    pub fn enabled(self, enabled: bool) -> Window {
        Window { enabled, ..self }
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

    fn draw_close_button(&self, _context: &mut WindowContext) -> bool {
        // let button_rect = Rect::new(
        //     context.window.position.x + context.window.size.x - 15.,
        //     context.window.position.y,
        //     20.,
        //     20.,
        // );
        // context.window.painter.draw_label(
        //     "X",
        //     Vec2::new(
        //         context.window.position.x + context.window.size.x - 10.,
        //         context.window.position.y + 3.,
        //     ),
        //     Some(context.style.title(context.focused)),
        //     unimplemented!(),
        //     unimplemented!(),
        // );
        // context.focused
        //     && button_rect.contains(context.input.mouse_position)
        //     && context.input.click_up
        unimplemented!()
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
                context.window.painter.draw_element_label(
                    &context.style.window_titlebar_style,
                    position,
                    &label,
                    ElementState {
                        focused,
                        clicked: false,
                        hovered: false,
                        selected: false,
                    },
                );
            }
            context.window.painter.draw_line(
                Vec2::new(position.x, position.y + style.title_height),
                Vec2::new(position.x + size.x, position.y + style.title_height),
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
