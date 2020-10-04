use crate::{
    types::{Rect, Vector2},
    ui::WindowContext,
    Id, Ui,
};

#[derive(Debug, Clone)]
pub struct Window {
    id: Id,
    position: Vector2,
    size: Vector2,
    close_button: bool,
    enabled: bool,
    movable: bool,
    titlebar: bool,
    label: Option<String>,
}

impl Window {
    pub fn new(id: Id, position: Vector2, size: Vector2) -> Window {
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
        let title_height = if self.titlebar {
            ui.style.title_height
        } else {
            0.
        };

        let context = ui.begin_window(
            self.id,
            None,
            self.position,
            self.size,
            title_height,
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

        context.window.draw_commands.clip(clip_rect);
        f(ui);

        let context = ui.get_active_window_context();
        context.window.draw_commands.clip(None);

        let opened = context.window.want_close == false;

        ui.end_window();

        opened
    }

    fn draw_close_button(&self, context: &mut WindowContext) -> bool {
        let button_rect = Rect::new(
            context.window.position.x + context.window.size.x - 15.,
            context.window.position.y,
            20.,
            20.,
        );
        context.window.draw_commands.draw_label(
            "X",
            Vector2::new(
                context.window.position.x + context.window.size.x - 10.,
                context.window.position.y + 3.,
            ),
            Some(context.global_style.title(context.focused)),
        );
        context.focused
            && button_rect.contains(context.input.mouse_position)
            && context.input.click_up
    }

    fn draw_window_frame(&self, context: &mut WindowContext) {
        let focused = context.focused;
        let style = context.global_style;
        let position = context.window.position;
        let size = context.window.size;

        context.window.draw_commands.draw_rect(
            Rect::new(position.x, position.y, size.x, size.y),
            style.window_border(focused),
            style.background(focused),
        );

        if self.titlebar {
            if let Some(label) = &self.label {
                context.window.draw_commands.draw_label(
                    &label,
                    Vector2::new(position.x + style.margin, position.y + style.margin),
                    context.global_style.title(focused),
                );
            }
            context.window.draw_commands.draw_line(
                Vector2::new(position.x, position.y + style.title_height),
                Vector2::new(position.x + size.x, position.y + style.title_height),
                style.window_border(focused),
            );
        }
    }
}

impl Ui {
    pub fn window<F: FnOnce(&mut Ui)>(
        &mut self,
        id: Id,
        position: Vector2,
        size: Vector2,
        f: F,
    ) -> bool {
        Window::new(id, position, size).ui(self, f)
    }
}
