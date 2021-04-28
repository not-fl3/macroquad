use crate::{
    math::{Rect, Vec2},
    ui::{ElementState, Id, Layout, Ui},
};

pub struct Tabbar<'a> {
    id: Id,
    size: Vec2,
    selected_tab: Option<usize>,
    tabs: &'a [&'a str],
}

impl Tabbar<'_> {
    pub fn new<'a>(id: Id, size: Vec2, tabs: &'a [&'a str]) -> Tabbar<'a> {
        Tabbar {
            id,
            size,
            tabs,
            selected_tab: None,
        }
    }

    pub fn selected_tab(self, selected_tab: Option<usize>) -> Self {
        Tabbar {
            selected_tab,
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui) -> u32 {
        let context = ui.get_active_window_context();

        let pos = context.window.cursor.fit(self.size, Layout::Vertical);

        let width = self.size.x as f32 / self.tabs.len() as f32;
        let selected = {
            let selected_mut = context.storage_u32.entry(self.id).or_insert(0);
            if let Some(n) = self.selected_tab {
                *selected_mut = n as u32;
            };
            *selected_mut
        };

        for (n, label) in self.tabs.iter().enumerate() {
            let rect = Rect::new(
                pos.x + width * n as f32 + 1.,
                pos.y,
                width - 2.,
                self.size.y,
            );
            let hovered = rect.contains(context.input.mouse_position);
            let selected = n as u32 == selected;

            if context.focused && hovered && context.input.click_up {
                *context.storage_u32.entry(self.id).or_insert(0) = n as u32;
            }

            context.window.painter.draw_element_background(
                &context.style.tabbar_style,
                rect.point(),
                rect.size(),
                ElementState {
                    focused: context.focused,
                    hovered,
                    clicked: hovered && context.input.is_mouse_down,
                    selected,
                },
            );

            let text_width = context
                .window
                .painter
                .element_size(&context.style.tabbar_style, label)
                .x;

            context.window.painter.draw_element_label(
                &context.style.tabbar_style,
                pos + Vec2::new(width * n as f32 + (width - text_width) / 2., 0.0),
                label,
                ElementState {
                    focused: context.focused,
                    hovered,
                    clicked: hovered && context.input.is_mouse_down,
                    selected,
                },
            );
        }
        context.storage_u32[&self.id]
    }
}

impl Ui {
    pub fn tabbar<'a>(&mut self, id: Id, size: Vec2, tabs: &'a [&'a str]) -> u32 {
        Tabbar::new(id, size, tabs).ui(self)
    }
}
