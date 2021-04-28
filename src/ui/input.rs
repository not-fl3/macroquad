use crate::math::Vec2;

pub use crate::ui::input_handler::KeyCode;

#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Char(char),
    KeyCode(KeyCode),
}

#[derive(Clone, Debug)]
pub struct InputCharacter {
    pub key: Key,
    pub modifier_shift: bool,
    pub modifier_ctrl: bool,
}

#[derive(Default, Clone)]
pub struct Input {
    pub(crate) mouse_position: Vec2,
    pub(crate) is_mouse_down: bool,
    pub(crate) click_down: bool,
    pub(crate) click_up: bool,
    pub(crate) mouse_wheel: Vec2,
    pub(crate) input_buffer: Vec<InputCharacter>,
    pub(crate) modifier_ctrl: bool,
    pub(crate) escape: bool,
    pub(crate) enter: bool,
    pub(crate) cursor_grabbed: bool,
    pub(crate) window_active: bool,
}

impl Input {
    pub fn is_mouse_down(&self) -> bool {
        self.is_mouse_down && self.cursor_grabbed == false && self.window_active
    }

    pub fn click_down(&self) -> bool {
        self.click_down && self.cursor_grabbed == false && self.window_active
    }

    pub fn click_up(&self) -> bool {
        self.click_up && self.cursor_grabbed == false && self.window_active
    }

    pub fn reset(&mut self) {
        self.modifier_ctrl = false;
        self.escape = false;
        self.enter = false;
        self.click_down = false;
        self.click_up = false;
        self.mouse_wheel = Vec2::new(0., 0.);
        self.input_buffer = vec![];
        self.window_active = false;
    }
}
