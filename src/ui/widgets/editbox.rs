use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, InputCharacter, Key, KeyCode, Layout, Ui},
};

pub struct Editbox<'a> {
    id: Id,
    size: Vec2,
    multiline: bool,
    select_all: bool,
    filter: Option<&'a dyn Fn(char) -> bool>,
    pos: Option<Vec2>,
    password: bool,
}

mod text_editor;

use text_editor::EditboxState;

const LEFT_MARGIN: f32 = 2.;

impl<'a> Editbox<'a> {
    pub fn new(id: Id, size: Vec2) -> Editbox<'a> {
        Editbox {
            id,
            size,
            filter: None,
            select_all: false,
            multiline: true,
            pos: None,
            password: false,
        }
    }

    pub fn multiline(self, multiline: bool) -> Self {
        Editbox { multiline, ..self }
    }

    pub fn select_all(self) -> Self {
        Editbox {
            select_all: true,
            ..self
        }
    }

    pub fn position(self, pos: Vec2) -> Self {
        Editbox {
            pos: Some(pos),
            ..self
        }
    }

    pub fn password(self, password: bool) -> Self {
        Editbox { password, ..self }
    }

    pub fn filter<'b>(self, filter: &'b dyn Fn(char) -> bool) -> Editbox<'b> {
        Editbox {
            id: self.id,
            pos: self.pos,
            multiline: self.multiline,
            select_all: self.select_all,
            size: self.size,
            password: self.password,
            filter: Some(filter),
        }
    }

    fn apply_keyboard_input(
        &self,
        context: &mut crate::Context,
        input_buffer: &mut Vec<InputCharacter>,
        clipboard: &mut dyn crate::ui::ClipboardObject,
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
                    if let Some(clipboard) = clipboard.get(context) {
                        if clipboard.len() != 0 {
                            if state.selection.is_some() {
                                state.delete_selected(text);
                            }

                            if let Some(filter) = &self.filter {
                                for character in clipboard.chars() {
                                    if filter(character) {
                                        state.insert_character(text, character);
                                    }
                                }
                            } else {
                                state.insert_string(text, clipboard);
                            }
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
                    let to_line_begin = state.find_line_begin(text) as i32;
                    state.move_cursor(text, -to_line_begin, modifier_shift);
                }
                InputCharacter {
                    key: Key::KeyCode(End),
                    modifier_shift,
                    ..
                } => {
                    let to_line_end = state.find_line_end(text) as i32;
                    state.move_cursor(text, to_line_end, modifier_shift);
                }
                InputCharacter {
                    key: Key::KeyCode(Up),
                    modifier_shift,
                    ..
                } => {
                    let to_line_begin = state.find_line_begin(text) as i32;
                    state.move_cursor(text, -to_line_begin, modifier_shift);
                    if state.cursor != 0 {
                        state.move_cursor(text, -1, modifier_shift);
                        let new_to_line_begin = state.find_line_begin(text) as i32;
                        let offset = to_line_begin.min(new_to_line_begin) - new_to_line_begin;
                        state.move_cursor(text, offset, modifier_shift);
                    }
                }
                InputCharacter {
                    key: Key::KeyCode(Down),
                    modifier_shift,
                    ..
                } => {
                    let to_line_begin = state.find_line_begin(text) as i32;
                    let to_line_end = state.find_line_end(text) as i32;

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

    pub fn ui(self, context: &mut crate::Context, ui: &mut Ui, text: &mut String) -> bool {
        let time = ui.time;

        let ctx = ui.get_active_window_context();

        let pos = self
            .pos
            .unwrap_or_else(|| ctx.window.cursor.fit(self.size, Layout::Vertical));

        let rect = Rect::new(pos.x, pos.y, self.size.x, self.size.y);

        let hovered = rect.contains(ctx.input.mouse_position);

        if ctx.input.click_down() && hovered {
            *ctx.input_focus = Some(self.id);
        }
        if ctx.input_focused(self.id) && ctx.input.click_down() && hovered == false {
            *ctx.input_focus = None;
        }

        let mut state = ctx
            .storage_any
            .get_or_default::<EditboxState>(hash!(self.id, "cursor"));

        // if text changed outside the selection range should be clamped
        state.clamp_selection(text);

        if self.select_all {
            state.select_all(text);
        }

        if let Some(selected) = state.selected_text(text) {
            *ctx.clipboard_selection = selected.to_owned();
        }
        // in case the string was updated outside of editbox
        if state.cursor > text.len() as u32 {
            state.cursor = text.len() as u32;
        }

        let input_focused = ctx.input_focus.map_or(false, |id| id == self.id) && ctx.focused;

        let is_tab_selected = ctx
            .tab_selector
            .register_selectable_widget(input_focused, ctx.input);
        if is_tab_selected {
            *ctx.input_focus = Some(self.id);
        }

        // reset selection state when lost focus
        if ctx.focused == false || input_focused == false {
            state.deselect();
            state.clicks_counter = 0;
        }
        if time - state.last_click_time > 3. * text_editor::DOUBLE_CLICK_TIME {
            state.clicks_counter = 0;
        }

        let mut edited = false;
        if ctx.focused && input_focused {
            edited = ctx.input.input_buffer.len() != 0;
            self.apply_keyboard_input(
                context,
                &mut ctx.input.input_buffer,
                &mut *ctx.clipboard,
                text,
                &mut state,
            );
        }
        // draw rect in parent window

        let text_color = ctx.style.editbox_style.text_color;

        ctx.window.painter.draw_element_background(
            &ctx.style.editbox_style,
            pos,
            self.size,
            ElementState {
                focused: ctx.focused,
                clicked: input_focused,
                ..Default::default()
            },
        );

        // start child window for nice scroll inside the rect

        let parent = ui.get_active_window_context();

        let parent_rect = parent.window.content_rect();

        parent.window.childs.push(self.id);
        let parent_id = Some(parent.window.id);

        let mut context = ui.begin_window(
            self.id,
            parent_id,
            pos,
            self.size + vec2(2., 2.),
            false,
            false,
        );

        let line_height = context.style.editbox_style.font_size as f32;

        let size = vec2(150., line_height * text.split('\n').count() as f32);

        // TODO: this is very weird hardcoded text margin
        let pos = context.window.cursor.fit(size, Layout::Free(vec2(2., 2.)));

        context.window.painter.clip(parent_rect);

        context.scroll_area();

        context.window.painter.clip(context.window.content_rect());

        let state = context
            .storage_any
            .get_or_default::<EditboxState>(hash!(self.id, "cursor"));

        let mut x = LEFT_MARGIN;
        let mut y = 0.;
        let mut clicked = false;

        for (n, character) in text.chars().chain(std::iter::once(' ')).enumerate() {
            let character = if character != '\n' && self.password {
                '*'
            } else {
                character
            };

            let font_size = context.style.editbox_style.font_size;
            if n == state.cursor as usize && input_focused {
                // caret
                context.window.painter.draw_rect(
                    Rect::new(pos.x + x, pos.y + y + 2., 2., font_size as f32 - 5.),
                    text_color,
                    None,
                );
            }
            // hack to draw a caret after text but do not bother with the extra symbol
            if n == text.len() {
                break;
            }

            let mut font = context.style.editbox_style.font.borrow_mut();
            let font_size = context.style.editbox_style.font_size;

            let mut advance = 1.5; // 1.5 - hack to make cursor on newlines visible

            if state.in_selected_range(n as u32) {
                let pos = pos + vec2(x, y);

                context.window.painter.draw_rect(
                    Rect::new(
                        pos.x,
                        pos.y,
                        context
                            .window
                            .painter
                            .character_advance(character, &font, font_size)
                            + 1.0,
                        font_size as f32 - 1.,
                    ),
                    None,
                    context.style.editbox_style.color_selected,
                );
            }

            if character != '\n' {
                let descent = font.descent(font_size as f32) as f32;
                let ascent = font.ascent(font_size as f32) as f32;
                let baseline = (ascent + descent) / 2.;

                advance = context
                    .window
                    .painter
                    .draw_character(
                        character,
                        pos + vec2(x, y + font_size as f32 - baseline),
                        text_color,
                        &mut *font,
                        font_size,
                    )
                    .unwrap_or(0.);
            }

            if clicked == false && hovered && context.input.is_mouse_down() && input_focused {
                let cursor_on_current_line =
                    (context.input.mouse_position.y - (pos.y + y + line_height / 2.)).abs()
                        < line_height / 2. + 0.1;
                let line_end = character == '\n' || n == text.len();
                let cursor_after_line_end = context.input.mouse_position.x > (pos.x + x);
                let clickable_character = character != '\n';
                let cursor_on_character =
                    (context.input.mouse_position.x - (pos.x + x)).abs() < advance / 2.;
                let last_character = n == text.len();
                let cursor_below_line =
                    (context.input.mouse_position.y - (pos.y + y + line_height)) > 0.;

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
                y += line_height;
                x = LEFT_MARGIN;
            }
        }

        if context.input.click_up() && input_focused {
            state.click_up(text);
        }

        let context = ui.get_active_window_context();

        context.window.painter.clip(None);

        ui.end_window();

        edited
    }
}

impl Ui {
    pub fn editbox(
        &mut self,
        context: &mut crate::Context,
        id: Id,
        size: Vec2,
        text: &mut String,
    ) -> bool {
        Editbox::new(id, size).ui(context, self, text)
    }
}
