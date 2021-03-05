//! Some hacks to emulate repeating flag from OS key events

use crate::ui::KeyCode;

#[derive(Default)]
pub(crate) struct KeyRepeat {
    character_this_frame: Option<KeyCode>,
    active_character: Option<KeyCode>,
    repeating_character: Option<KeyCode>,
    pressed_time: f32,
}

impl KeyRepeat {
    pub fn new() -> KeyRepeat {
        KeyRepeat::default()
    }

    // allow the key to be pressed once and than to be pressed all the time but only after some delay
    pub(crate) fn add_repeat_gap(&mut self, character: KeyCode, _time: f32) -> bool {
        self.character_this_frame = Some(character);

        self.active_character.is_none()
            || self.active_character != self.character_this_frame
            || self.repeating_character == self.character_this_frame
    }

    pub(crate) fn new_frame(&mut self, time: f32) {
        let character_this_frame = self.character_this_frame.take();

        if character_this_frame == self.active_character && time - self.pressed_time > 0.5 {
            self.repeating_character = self.active_character;
        }

        if character_this_frame != self.active_character {
            self.active_character = character_this_frame;
            self.pressed_time = time;
            self.repeating_character = None;
        }
    }
}
