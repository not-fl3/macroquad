use crate::{get_context, drawing::MegauiDrawContext};
use miniquad::{KeyCode, MouseButton, KeyMods};
use megaui::InputHandler;

impl miniquad::EventHandlerFree for MegauiDrawContext {
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.ui.mouse_move((x, y));
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.ui.mouse_wheel(x, -y);
    }
    fn mouse_button_down_event(&mut self, _: MouseButton, x: f32, y: f32) {
        self.ui.mouse_down((x, y));
    }
    fn mouse_button_up_event(&mut self, _: MouseButton, x: f32, y: f32) {
        self.ui.mouse_up((x, y));
    }

    fn char_event(&mut self, character: char, modifiers: KeyMods, _repeat: bool) {
        self.ui.char_event(character, modifiers.shift, modifiers.ctrl);
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, _repeat: bool) {
        fn convert_keycode(keycode: KeyCode) -> Option<megaui::KeyCode> {
            Some(match keycode {
                KeyCode::Up => megaui::KeyCode::Up,
                KeyCode::Down => megaui::KeyCode::Down,
                KeyCode::Right => megaui::KeyCode::Right,
                KeyCode::Left => megaui::KeyCode::Left,
                KeyCode::Home => megaui::KeyCode::Home,
                KeyCode::End => megaui::KeyCode::End,
                KeyCode::Delete => megaui::KeyCode::Delete,
                KeyCode::Backspace => megaui::KeyCode::Backspace,
                KeyCode::Enter => megaui::KeyCode::Enter,
                KeyCode::Tab => megaui::KeyCode::Tab,
                KeyCode::Z => megaui::KeyCode::Z,
                KeyCode::Y => megaui::KeyCode::Y,
                KeyCode::C => megaui::KeyCode::C,
                KeyCode::X => megaui::KeyCode::X,
                KeyCode::V => megaui::KeyCode::V,
                KeyCode::A => megaui::KeyCode::A,
                _ => return None,
            })
        }

        if let Some(key) = convert_keycode(keycode) {
            self.ui.key_down(key, modifiers.shift, modifiers.ctrl);
        }
    }

    fn update(&mut self) {}

    fn draw(&mut self) {}
}

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
    get_context().draw_context.ui_mut().set_style(style);
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
    .ui(context.ui_mut(), f)
}
