use crate::{
    math::{vec2, Rect},
    ui::{ElementState, Layout, Ui},
};

pub struct ProgressBar<'a> {
    label: &'a str,
    label_width: Option<f32>,
}

impl<'a> ProgressBar<'a> {
    pub const fn new() -> Self {
        ProgressBar {
            label: "",
            label_width: None,
        }
    }

    pub const fn label<'b>(self, label: &'b str) -> ProgressBar<'b> {
        ProgressBar { label, ..self }
    }

    pub const fn label_width(self, width: f32) -> Self {
        Self {
            label_width: Some(width),
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui, progress: f32, bar_text: &str) {
        let mut context = ui.get_active_window_context();

        let size = vec2(
            context.window.cursor.area.w - context.style.margin * 3. - context.window.cursor.ident,
            19.,
        );

        let pos = context.window.cursor.fit(size, Layout::Vertical);
        let label_width = self.label_width.unwrap_or(100.);
        let bar_width = size.x - label_width;
        let bar_progress_width = progress.clamp(0., 1.) * bar_width;

        let bar_background = Rect::new(pos.x, pos.y, bar_width, 20.);
        let bar_progress = Rect::new(pos.x, pos.y, bar_progress_width, 20.);

        context.register_click_intention(bar_background);

        // background bar
        context.window.painter.draw_rect(
            bar_background,
            None,
            context.style.progress_bar_style.color(ElementState {
                focused: context.focused,
                hovered: false,
                clicked: false,
                selected: false,
            }),
        );
        // progress bar
        context.window.painter.draw_rect(
            bar_progress,
            None,
            context.style.progress_bar_style.color(ElementState {
                focused: context.focused,
                hovered: true,
                clicked: false,
                selected: false,
            }),
        );
        // label
        context.window.painter.draw_element_label(
            &context.style.label_style,
            vec2(pos.x + bar_width + 10., pos.y + 2.),
            self.label,
            ElementState {
                focused: context.focused,
                ..Default::default()
            },
        );
        // bar text
        context.window.painter.draw_element_content(
            &context.style.label_style,
            vec2(pos.x, pos.y),
            vec2(bar_width, 20.),
            &bar_text.into(),
            ElementState {
                focused: context.focused,
                ..Default::default()
            },
        );
    }
}

impl Ui {
    pub fn progress_bar(&mut self, label: &str, progress: f32) {
        ProgressBar::new().label(label).ui(self, progress, "");
    }
}
