use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, Layout, Ui, UiContent},
};

pub struct Tabbar<'a, 'b> {
    id: Id,
    size: Vec2,
    selected_tab: Option<&'b mut u32>,
    tabs: &'a [&'a str],
}

impl<'a, 'b> Tabbar<'a, 'b> {
    pub fn new(id: Id, size: Vec2, tabs: &'a [&'a str]) -> Tabbar<'a, 'b> {
        Tabbar {
            id,
            size,
            tabs,
            selected_tab: None,
        }
    }

    pub fn selected_tab<'c>(self, selected_tab: Option<&'c mut u32>) -> Tabbar<'a, 'c> {
        Tabbar {
            id: self.id,
            selected_tab,
            size: self.size,
            tabs: self.tabs,
        }
    }

    pub fn ui(mut self, ui: &mut Ui) -> u32 {
        let context = ui.get_active_window_context();

        let pos = context.window.cursor.fit(self.size, Layout::Vertical);

        let width = self.size.x as f32 / self.tabs.len() as f32;

        let selected = *self
            .selected_tab
            .as_deref()
            .unwrap_or_else(|| context.storage_u32.entry(self.id).or_insert(0));

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
                let id = self.id;
                let selected_mut = self
                    .selected_tab
                    .as_deref_mut()
                    .unwrap_or_else(|| context.storage_u32.entry(id).or_insert(0));

                *selected_mut = n as u32;
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

            context.window.painter.draw_element_content(
                &context.style.tabbar_style,
                pos + vec2(width * n as f32, 0.0),
                vec2(width, self.size.y),
                &UiContent::Label((*label).into()),
                ElementState {
                    focused: context.focused,
                    hovered,
                    clicked: hovered && context.input.is_mouse_down,
                    selected,
                },
            );
        }

        *self
            .selected_tab
            .as_deref()
            .unwrap_or_else(|| context.storage_u32.entry(self.id).or_insert(0))
    }
}

impl Ui {
    pub fn tabbar<'a>(&mut self, id: Id, size: Vec2, tabs: &'a [&'a str]) -> u32 {
        Tabbar::new(id, size, tabs).ui(self)
    }
}
