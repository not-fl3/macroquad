use crate::types::Vector2;

pub use crate::input_handler::KeyCode;

#[derive(Clone, Debug)]
pub enum Key {
    Char(char),
    KeyCode(KeyCode)
}

#[derive(Clone, Debug)]
pub struct InputCharacter {
    pub key: Key,
    pub modifier_shift: bool,
    pub modifier_ctrl: bool
}

#[derive(Default, Clone)]
pub struct Input {
    pub mouse_position: Vector2,
    pub is_mouse_down: bool,
    pub click_down: bool,
    pub click_up: bool,
    pub mouse_wheel: Vector2,
    pub input_buffer: Vec<InputCharacter>,
    pub cursor_grabbed: bool,
}

impl Input {
    pub fn is_mouse_down(&self) -> bool {
        self.is_mouse_down && self.cursor_grabbed == false
    }

    pub fn click_down(&self) -> bool {
        self.click_down && self.cursor_grabbed == false
    }

    pub fn click_up(&self) -> bool {
        self.click_up && self.cursor_grabbed == false
    }

    pub fn reset(&mut self) {
        self.click_down = false;
        self.click_up = false;
        self.mouse_wheel = Vector2::new(0., 0.);
        self.input_buffer = vec![];
    }
}
