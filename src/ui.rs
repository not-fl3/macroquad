
use crate::get_context;

pub struct ClipboardObject;

impl megaui::ClipboardObject for ClipboardObject {
    fn get(&self) -> Option<String> {
        let context = get_context();

        miniquad::clipboard::get(&mut context.quad_context)
    }

    fn set(&mut self, data: &str) {
        let context = get_context();

        miniquad::clipboard::set(&mut context.quad_context, data)
    }
}

pub struct WindowParams {
    pub label: String,
    pub movable: bool,
    pub close_button: bool,
    pub titlebar: bool,
}

impl Default for WindowParams {
    fn default() -> WindowParams {
        WindowParams {
            label: "".to_string(),
            movable: true,
            close_button: false,
            titlebar: true,
        }
    }
}

pub fn set_ui_style(style: megaui::Style) {
    get_context().draw_context.ui.set_style(style);
}

pub fn draw_window<F: FnOnce(&mut megaui::Ui)>(
    id: megaui::Id,
    position: glam::Vec2,
    size: glam::Vec2,
    params: impl Into<Option<WindowParams>>,
    f: F,
) -> bool {
    let context = &mut get_context().draw_context;
    let params = params.into();

    megaui::widgets::Window::new(
        id,
        megaui::Vector2::new(position.x(), position.y()),
        megaui::Vector2::new(size.x(), size.y()),
    )
    .label(params.as_ref().map_or("", |params| &params.label))
    .titlebar(params.as_ref().map_or(true, |params| params.titlebar))
    .movable(params.as_ref().map_or(true, |params| params.movable))
    .close_button(params.as_ref().map_or(false, |params| params.close_button))
    .ui(&mut context.ui, f)
}
