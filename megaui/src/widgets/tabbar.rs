use crate::{
    draw_command::Aligment,
    types::{Color, Rect, Vector2},
    Id, Layout, Ui,
};

pub struct Tabbar {
    id: Id,
    position: Vector2,
    size: Vector2,
    tabs: &'static [&'static str],
}

impl Tabbar {
    pub fn new(id: Id, position: Vector2, size: Vector2, tabs: &'static [&'static str]) -> Tabbar {
        Tabbar {
            id,
            position,
            size,
            tabs,
        }
    }

    pub fn ui(self, ui: &mut Ui) -> u32 {
        let context = ui.get_active_window_context();

        let pos = context
            .window
            .cursor
            .fit(self.size, Layout::Free(self.position));

        let width = self.size.x as f32 / self.tabs.len() as f32;
        let selected = *context.storage_u32.entry(self.id).or_insert(0);

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
            context.window.draw_commands.draw_rect(
                rect,
                None,
                context.global_style.tabbar_background(
                    context.focused,
                    selected,
                    hovered,
                    hovered && context.input.is_mouse_down,
                ),
            );

            context.window.draw_commands.draw_label(
                label,
                pos + Vector2::new(
                    width * n as f32 + width / 2.,
                    context.global_style.margin_button + 2.,
                ),
                (
                    if selected {
                        Color::new(1., 1., 1., 1.)
                    } else {
                        context.global_style.text(context.focused)
                    },
                    Aligment::Center,
                ),
            );
        }
        context.storage_u32[&self.id]
    }
}

impl Ui {
    pub fn tabbar(
        &mut self,
        id: Id,
        position: Vector2,
        size: Vector2,
        tabs: &'static [&'static str],
    ) -> u32 {
        Tabbar::new(id, position, size, tabs).ui(self)
    }
}
