use crate::{
    hash,
    types::{Rect, Vector2},
    ui::{InputCharacter, Key, KeyCode},
    Id, Layout, Ui,
};

pub struct Editbox<'a> {
    id: Id,
    size: Vector2,
    multiline: bool,
    filter: Option<&'a dyn Fn(char) -> bool>,
    pos: Option<Vector2>,
    line_height: f32,
}

mod text_editor;

pub use text_editor::EditboxState;

const LEFT_MARGIN: f32 = 2.;
const N_SPACES_IN_TAB: usize = 4;

impl<'a> Editbox<'a> {
    pub fn new(id: Id, size: Vector2) -> Editbox<'a> {
        Editbox {
            id,
            size,
            filter: None,
            multiline: true,
            pos: None,
            line_height: 14.0,
        }
    }

    pub fn multiline(self, multiline: bool) -> Self {
        Editbox { multiline, ..self }
    }

    pub fn position(self, pos: Vector2) -> Self {
        Editbox {
            pos: Some(pos),
            ..self
        }
    }

    pub fn line_height(self, line_height: f32) -> Self {
        Self {
            line_height,
            ..self
        }
    }

    pub fn filter<'b>(self, filter: &'b dyn Fn(char) -> bool) -> Editbox<'b> {
        Editbox {
            id: self.id,
            line_height: self.line_height,
            pos: self.pos,
            multiline: self.multiline,
            size: self.size,
            filter: Some(filter),
        }
    }

    fn apply_keyboard_input(
        &self,
        input_buffer: &mut Vec<InputCharacter>,
        clipboard: &mut dyn crate::ClipboardObject,
        text: &mut String,
        state: &mut EditboxState,
    ) {
        for character in input_buffer.drain(0..) {
            use KeyCode::*;

            match character {
                InputCharacter {
                    key: Key::Char(_),
                    modifier_ctrl: true,
                    ..
                } => {}
                InputCharacter {
                    key: Key::Char(character),
                    modifier_ctrl: false,
                    ..
                } => {
                    if character != 13 as char
                        && character != 10 as char
                        && character.is_ascii()
                        && self.filter.as_ref().map_or(true, |f| f(character))
                    {
                        if state.selection.is_some() {
                            state.delete_selected(text);
                        }
                        state.insert_character(text, character);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Z),
                    modifier_ctrl: true,
                    ..
                } => {
                    state.undo(text);
                }
                InputCharacter {
                    key: Key::KeyCode(Y),
                    modifier_ctrl: true,
                    ..
                } => {
                    state.redo(text);
                }
                InputCharacter {
                    key: Key::KeyCode(X),
                    modifier_ctrl: true,
                    ..
                } => {
                    state.delete_selected(text);
                }
                InputCharacter {
                    key: Key::KeyCode(V),
                    modifier_ctrl: true,
                    ..
                } => {
                    if let Some(clipboard) = clipboard.get() {
                        if clipboard.len() != 0 {
                            if state.selection.is_some() {
                                state.delete_selected(text);
                            }

                            state.insert_string(text, clipboard);
                        }
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(A),
                    modifier_ctrl: true,
                    ..
                } => {
                    state.select_all(text);
                }
                InputCharacter {
                    key: Key::KeyCode(Enter),
                    ..
                } => {
                    if self.multiline {
                        state.insert_character(text, '\n');
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Tab),
                    modifier_shift: false,
                    ..
                } => {
                    state.insert_string(text, " ".repeat(N_SPACES_IN_TAB));
                }
                InputCharacter {
                    key: Key::KeyCode(Tab),
                    modifier_shift: true,
                    ..
                } => {
                    for _ in 0..N_SPACES_IN_TAB {
                        let cursor = state.cursor as usize;
                        if cursor != 0  {
                            let current_char = text.chars().nth(cursor - 1).unwrap();
                            if current_char == ' ' {
                                state.delete_current_character(text);
                            } else {
                                return
                            }
                        } else {
                            return
                        }
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Backspace),
                    ..
                } => {
                    if state.selection.is_none() {
                        state.delete_current_character(text);
                    } else {
                        state.delete_selected(text);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Delete),
                    ..
                } => {
                    if state.selection.is_none() {
                        state.delete_next_character(text);
                    } else {
                        state.delete_selected(text);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Right),
                    modifier_shift,
                    modifier_ctrl,
                } => {
                    if modifier_ctrl {
                        state.move_cursor_next_word(text, modifier_shift);
                    } else {
                        state.move_cursor(text, 1, modifier_shift);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Left),
                    modifier_shift,
                    modifier_ctrl,
                } => {
                    if modifier_ctrl {
                        state.move_cursor_prev_word(text, modifier_shift);
                    } else {
                        state.move_cursor(text, -1, modifier_shift);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Home),
                    modifier_shift,
                    ..
                } => {
                    let to_line_begin = state.find_line_begin(&text) as i32;
                    state.move_cursor(text, -to_line_begin, modifier_shift);
                }
                InputCharacter {
                    key: Key::KeyCode(End),
                    modifier_shift,
                    ..
                } => {
                    let to_line_end = state.find_line_end(&text) as i32;
                    state.move_cursor(text, to_line_end, modifier_shift);
                }
                InputCharacter {
                    key: Key::KeyCode(Up),
                    modifier_shift,
                    ..
                } => {
                    let to_line_begin = state.find_line_begin(&text) as i32;
                    state.move_cursor(text, -to_line_begin, modifier_shift);
                    if state.cursor != 0 {
                        state.move_cursor(text, -1, modifier_shift);
                        let new_to_line_begin = state.find_line_begin(&text) as i32;
                        let offset = to_line_begin.min(new_to_line_begin) - new_to_line_begin;
                        state.move_cursor(text, offset, modifier_shift);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Down),
                    modifier_shift,
                    ..
                } => {
                    let to_line_begin = state.find_line_begin(&text) as i32;
                    let to_line_end = state.find_line_end(&text) as i32;

                    state.move_cursor(text, to_line_end, modifier_shift);
                    if text.len() != 0 && state.cursor < text.len() as u32 - 1 {
                        state.move_cursor(text, 1, modifier_shift);
                        state.move_cursor_within_line(text, to_line_begin, modifier_shift);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn ui(self, ui: &mut Ui, text: &mut String) -> bool {
        // TODO: change API to accept real time
        let time = ui.frame as f32 / 60.;

        let context = ui.get_active_window_context();

        let pos = self
            .pos
            .unwrap_or_else(|| context.window.cursor.fit(self.size, Layout::Vertical));

        let rect = Rect::new(pos.x, pos.y, self.size.x, self.size.y);

        let hovered = rect.contains(context.input.mouse_position);

        if context.input.click_down() && hovered {
            context.window.input_focus = Some(self.id);
        }
        if context.window.input_focused(self.id) && context.input.click_down() && hovered == false {
            context.window.input_focus = None;
        }

        let mut state = context
            .storage_any
            .get_or_default::<EditboxState>(hash!(self.id, "cursor"));

        if let Some(selected) = state.selected_text(text) {
            *context.clipboard_selection = selected.to_owned();
        }
        // in case the string was updated outside of editbox
        if state.cursor > text.len() as u32 {
            state.cursor = text.len() as u32;
        }

        let input_focused = context.window.input_focused(self.id) && context.focused;

        // reset selection state when lost focus
        if context.focused == false || input_focused == false {
            state.deselect();
            state.clicks_counter = 0;
        }
        if time - state.last_click_time > 3. * text_editor::DOUBLE_CLICK_TIME {
            state.clicks_counter = 0;
        }

        let mut edited = false;
        if context.focused && input_focused {
            edited = context.input.input_buffer.len() != 0;
            self.apply_keyboard_input(
                &mut context.input.input_buffer,
                &mut *context.clipboard,
                text,
                &mut state,
            );
        }

        let color = context.global_style.text(context.focused);

        // draw rect in parent window

        context.window.draw_commands.draw_rect(
            rect,
            context.global_style.editbox_background(context.focused),
            None,
        );

        // start child window for nice scroll inside the rect

        let parent = ui.get_active_window_context();

        let parent_rect = parent.window.content_rect();

        parent.window.childs.push(self.id);
        let parent_id = Some(parent.window.id);

        let mut context = ui.begin_window(self.id, parent_id, pos, self.size, 0., false);

        let size = Vector2::new(150., self.line_height * text.split('\n').count() as f32);

        let pos = context
            .window
            .cursor
            .fit(size, Layout::Free(Vector2::new(5., 5.)));

        context.window.draw_commands.clip(parent_rect);

        context.scroll_area();

        context
            .window
            .draw_commands
            .clip(context.window.content_rect());

        let state = context
            .storage_any
            .get_or_default::<EditboxState>(hash!(self.id, "cursor"));

        let mut x = LEFT_MARGIN;
        let mut y = 0.;
        let mut clicked = false;

        for n in 0..text.len() + 1 {
            let character = text.chars().nth(n).unwrap_or(' ');
            if n == state.cursor as usize {
                context.window.draw_commands.draw_rect(
                    Rect::new(pos.x + x, pos.y + y - 2., 2., 13.),
                    context
                        .global_style
                        .editbox_cursor(context.focused, input_focused),
                    None,
                );
            }
            let mut advance = 1.5; // 1.5 - hack to make cursor on newlines visible
            if character != '\n' {
                advance = context
                    .window
                    .draw_commands
                    .draw_character(character, pos + Vector2::new(x, y), color)
                    .unwrap_or(0.);
            }
            if state.in_selected_range(n as u32) {
                let pos = pos + Vector2::new(x, y);

                context.window.draw_commands.draw_rect(
                    Rect::new(pos.x, pos.y - 2., advance, 13.),
                    None,
                    context.global_style.selection_background(context.focused),
                );
            }

            if clicked == false && hovered && context.input.is_mouse_down() && input_focused {
                let cursor_on_current_line =
                    (context.input.mouse_position.y - (pos.y + y + self.line_height / 2.)).abs()
                        < self.line_height / 2. + 0.1;
                let line_end = character == '\n' || n == text.len();
                let cursor_after_line_end = context.input.mouse_position.x > (pos.x + x);
                let clickable_character = character != '\n';
                let cursor_on_character =
                    (context.input.mouse_position.x - (pos.x + x)).abs() < advance / 2.;
                let last_character = n == text.len();
                let cursor_below_line =
                    (context.input.mouse_position.y - (pos.y + y + self.line_height)) > 0.;

                if (cursor_on_current_line && line_end && cursor_after_line_end)
                    || (cursor_on_current_line && clickable_character && cursor_on_character)
                    || (last_character && cursor_below_line)
                {
                    clicked = true;

                    if context.input.click_down() {
                        state.click_down(time, text, n as u32);
                    } else {
                        state.click_move(text, n as u32);
                    }
                }
            }

            x += advance;
            if character == '\n' && self.multiline {
                y += self.line_height;
                x = LEFT_MARGIN;
            }
        }

        if context.input.click_up() && input_focused {
            state.click_up(text);
        }

        let context = ui.get_active_window_context();

        context.window.draw_commands.clip(None);

        ui.end_window();

        edited
    }
}

impl Ui {
    pub fn editbox(&mut self, id: Id, size: Vector2, text: &mut String) -> bool {
        Editbox::new(id, size).ui(self, text)
    }
}
