trait Command {
    fn apply(&self, text_cursor: &mut u32, text: &mut Vec<char>);
    fn unapply(&self, text_cursor: &mut u32, text: &mut Vec<char>);
}

struct InsertCharacter {
    character: char,
    cursor: u32,
}

impl InsertCharacter {
    fn new(editor: &EditboxState, _text: &mut Vec<char>, character: char) -> InsertCharacter {
        InsertCharacter {
            cursor: editor.cursor,
            character,
        }
    }
}

impl Command for InsertCharacter {
    fn apply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor;
        if self.cursor <= text.len() as u32 {
            text.insert(self.cursor as usize, self.character);
        }
        *text_cursor += 1;
    }
    fn unapply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor;
        if self.cursor < text.len() as u32 {
            text.remove(self.cursor as usize);
        }
    }
}

struct InsertString {
    data: Vec<char>,
    cursor: u32,
}

impl InsertString {
    fn new(editor: &EditboxState, _text: &mut Vec<char>, data: Vec<char>) -> InsertString {
        InsertString {
            cursor: editor.cursor,
            data,
        }
    }
}

impl Command for InsertString {
    fn apply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor;
        if self.cursor <= text.len() as u32 {
            text.splice(
                self.cursor as usize..self.cursor as usize,
                self.data.clone(),
            );
        }
        *text_cursor += self.data.len() as u32;
    }

    fn unapply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor;
        if self.cursor < text.len() as u32 {
            let end = (self.cursor as usize + self.data.len()).min(text.len());
            text.drain(self.cursor as usize..end);
        }
    }
}

struct DeleteCharacter {
    character: char,
    cursor: u32,
}

impl DeleteCharacter {
    fn new(editor: &EditboxState, text: &mut Vec<char>) -> Option<DeleteCharacter> {
        let character = text.get(editor.cursor as usize).copied();

        character.map(|character| DeleteCharacter {
            cursor: editor.cursor,
            character,
        })
    }
}

impl Command for DeleteCharacter {
    fn apply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor;
        if self.cursor < text.len() as u32 {
            text.remove(self.cursor as usize);
        }
    }

    fn unapply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        *text_cursor = self.cursor + 1;
        if self.cursor <= text.len() as u32 {
            text.insert(self.cursor as usize, self.character);
        }
    }
}

struct DeleteRange {
    range: (u32, u32),
    data: Vec<char>,
}

impl DeleteRange {
    fn new(text: &mut Vec<char>, (start, end): (u32, u32)) -> DeleteRange {
        let min = start.min(end) as usize;
        let max = start.max(end) as usize;

        DeleteRange {
            data: text[min..max].to_vec(),
            range: (start, end),
        }
    }
}

impl Command for DeleteRange {
    fn apply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        let (start, end) = self.range;
        let min = start.min(end) as usize;
        let max = start.max(end) as usize;

        text.drain(min..max);

        *text_cursor = min as u32;
    }

    fn unapply(&self, text_cursor: &mut u32, text: &mut Vec<char>) {
        let (start, end) = self.range;
        let start = start.min(end) as usize;
        text.splice(start..start, self.data.clone());
        *text_cursor = start as u32;
    }
}

#[derive(Debug)]
pub enum ClickState {
    None,
    SelectingChars { selection_begin: u32 },
    SelectingWords { selected_word: (u32, u32) },
    SelectingLines { selected_line: (u32, u32) },
    Selected,
}
impl Default for ClickState {
    fn default() -> ClickState {
        ClickState::None
    }
}
pub const DOUBLE_CLICK_TIME: f32 = 0.5;

#[derive(Default)]
pub struct EditboxState {
    pub cursor: u32,
    pub click_state: ClickState,
    pub clicks_counter: u32,
    pub current_click: u32,
    pub last_click_time: f32,
    pub last_click: u32,
    pub selection: Option<(u32, u32)>,
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl EditboxState {
    pub fn clamp_selection<'a>(&mut self, text: &'a Vec<char>) {
        if let Some((ref mut start, ref mut end)) = &mut self.selection {
            if *start >= text.len() as u32 {
                *start = text.len() as _;
            }
            if *end >= text.len() as u32 {
                *end = text.len() as _;
            }
        }
    }

    pub fn selected_text<'a>(&self, text: &'a Vec<char>) -> Option<&'a [char]> {
        if let Some((start, end)) = self.selection {
            let min = start.min(end) as usize;
            let max = start.max(end) as usize;

            assert!(min <= max);
            assert!(max <= text.len());

            Some(&text[min..max])
        } else {
            None
        }
    }
    pub const fn in_selected_range(&self, cursor: u32) -> bool {
        match self.selection {
            Some((start, end)) if start < end => cursor >= start && cursor < end,
            Some((end, start)) => cursor >= start && cursor < end,
            _ => false,
        }
    }
    pub fn find_line_begin(&self, text: &Vec<char>) -> u32 {
        let mut line_position = 0;
        let mut cursor_tmp = self.cursor;

        while cursor_tmp > 0 && text.get(cursor_tmp as usize - 1).copied().unwrap_or('x') != '\n' {
            cursor_tmp -= 1;
            line_position += 1;
        }
        line_position
    }

    pub fn find_line_end(&self, text: &Vec<char>) -> u32 {
        let mut cursor_tmp = self.cursor;
        while cursor_tmp < text.len() as u32
            && text.get(cursor_tmp as usize).copied().unwrap_or('x') != '\n'
        {
            cursor_tmp += 1;
        }

        cursor_tmp - self.cursor
    }

    pub const fn word_delimiter(character: char) -> bool {
        character == ' '
            || character == '('
            || character == ')'
            || character == ';'
            || character == '\"'
    }

    pub fn find_word_begin(&self, text: &Vec<char>, cursor: u32) -> u32 {
        let mut cursor_tmp = cursor;
        let mut offset = 0;

        while cursor_tmp > 0 {
            let current_char = text.get(cursor_tmp as usize - 1).copied().unwrap_or(' ');
            if Self::word_delimiter(current_char) || current_char == '\n' {
                break;
            }
            offset += 1;
            cursor_tmp -= 1;
        }
        offset
    }

    pub fn find_word_end(&self, text: &Vec<char>, cursor: u32) -> u32 {
        let mut cursor_tmp = cursor;
        let mut offset = 0;
        let mut space_skipping = false;

        while cursor_tmp < text.len() as u32 {
            let current_char = text.get(cursor_tmp as usize).copied().unwrap_or(' ');
            if Self::word_delimiter(current_char) || current_char == '\n' {
                space_skipping = true;
            }
            if space_skipping && Self::word_delimiter(current_char) == false {
                break;
            }
            cursor_tmp += 1;
            offset += 1;
        }
        offset
    }

    pub fn insert_character(&mut self, text: &mut Vec<char>, character: char) {
        self.redo_stack.clear();

        self.selection = None;

        let insert_command = InsertCharacter::new(self, text, character);
        insert_command.apply(&mut self.cursor, text);
        self.undo_stack.push(Box::new(insert_command));
    }

    pub fn insert_string(&mut self, text: &mut Vec<char>, string: Vec<char>) {
        self.redo_stack.clear();

        self.selection = None;

        let insert_command = InsertString::new(self, text, string.to_owned());
        insert_command.apply(&mut self.cursor, text);
        self.undo_stack.push(Box::new(insert_command));
    }

    pub fn delete_selected(&mut self, text: &mut Vec<char>) {
        self.redo_stack.clear();

        if let Some(range) = self.selection {
            let delete_command = DeleteRange::new(text, range);
            delete_command.apply(&mut self.cursor, text);
            self.undo_stack.push(Box::new(delete_command));
        }
        self.selection = None;
    }

    pub fn delete_next_character(&mut self, text: &mut Vec<char>) {
        self.redo_stack.clear();

        if let Some(delete_command) = DeleteCharacter::new(self, text) {
            delete_command.apply(&mut self.cursor, text);
            self.undo_stack.push(Box::new(delete_command));
        }
    }

    pub fn delete_current_character(&mut self, text: &mut Vec<char>) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.delete_next_character(text);
        }
    }

    pub fn move_cursor_next_word(&mut self, text: &Vec<char>, shift: bool) {
        let next_word = self.find_word_end(text, self.cursor + 1) + 1;
        self.move_cursor(text, next_word as i32, shift);
    }

    pub fn move_cursor_prev_word(&mut self, text: &Vec<char>, shift: bool) {
        if self.cursor > 1 {
            let prev_word = self.find_word_begin(text, self.cursor - 1) + 1;
            self.move_cursor(text, -(prev_word as i32), shift);
        }
    }

    pub fn move_cursor(&mut self, text: &Vec<char>, dx: i32, shift: bool) {
        let start_cursor = self.cursor;
        let mut end_cursor = start_cursor;

        if self.cursor as i32 + dx <= text.len() as i32 && self.cursor as i32 + dx >= 0 {
            end_cursor = (self.cursor as i32 + dx) as u32;
            self.cursor = end_cursor;
        }

        if shift == false {
            self.selection = None;
        }
        if shift {
            match &mut self.selection {
                None => self.selection = Some((start_cursor, end_cursor)),
                Some((_, ref mut end)) => {
                    *end = end_cursor;
                }
            }
        }
    }

    pub fn move_cursor_within_line(&mut self, text: &Vec<char>, dx: i32, shift: bool) {
        assert!(dx >= 0, "not implemented");

        for _ in 0..dx {
            if text.get(self.cursor as usize).copied().unwrap_or('x') == '\n'
                || self.cursor == text.len() as u32
            {
                break;
            }
            self.move_cursor(text, 1, shift);
        }
    }

    pub fn select_all(&mut self, text: &Vec<char>) {
        self.selection = Some((0, text.len() as u32));
        self.click_state = ClickState::None;
    }

    pub fn deselect(&mut self) {
        self.click_state = ClickState::None;
        self.selection = None;
    }

    pub fn select_word(&mut self, text: &Vec<char>) -> (u32, u32) {
        let to_word_begin = self.find_word_begin(text, self.cursor);
        let to_word_end = self.find_word_end(text, self.cursor);
        let new_selection = (self.cursor - to_word_begin, self.cursor + to_word_end);

        self.selection = Some(new_selection);
        new_selection
    }

    pub fn select_line(&mut self, text: &Vec<char>) -> (u32, u32) {
        let to_line_begin = self.find_line_begin(text);
        let to_line_end = self.find_line_end(text);
        let new_selection = (self.cursor - to_line_begin, self.cursor + to_line_end);

        self.selection = Some(new_selection);
        new_selection
    }

    pub fn click_down(&mut self, time: f32, text: &Vec<char>, cursor: u32) {
        self.current_click = cursor;

        if self.last_click == self.current_click && time - self.last_click_time < DOUBLE_CLICK_TIME
        {
            self.clicks_counter += 1;
            match self.clicks_counter % 3 {
                0 => {
                    self.deselect();
                    self.click_state = ClickState::None;
                }
                1 => {
                    let selected_word = self.select_word(text);
                    self.click_state = ClickState::SelectingWords { selected_word };
                }
                2 => {
                    let selected_line = self.select_line(text);
                    self.click_state = ClickState::SelectingLines { selected_line }
                }
                _ => unreachable!(),
            }
        } else {
            self.clicks_counter = 0;
            if let ClickState::None | ClickState::Selected = self.click_state {
                self.click_state = ClickState::SelectingChars {
                    selection_begin: cursor,
                };
                self.selection = Some((cursor, cursor));
            } else {
                self.click_state = ClickState::None;
                self.selection = None;
                self.cursor = cursor;
            }
        }

        self.last_click_time = time;
    }

    pub fn click_move(&mut self, text: &Vec<char>, cursor: u32) {
        self.cursor = cursor;

        if self.cursor != self.last_click {
            self.clicks_counter = 0;
        }

        match self.click_state {
            ClickState::SelectingChars { selection_begin } => {
                self.selection = Some((selection_begin, cursor));
            }
            ClickState::SelectingWords {
                selected_word: (from, to),
            } => {
                if cursor < from {
                    let word_begin = self.cursor - self.find_word_begin(text, self.cursor);
                    self.selection = Some((word_begin, to));
                    self.cursor = word_begin;
                } else if cursor > to {
                    let word_end = self.cursor + self.find_word_end(text, self.cursor);
                    self.selection = Some((from, word_end));
                    self.cursor = word_end;
                } else {
                    self.selection = Some((from, to));
                    self.cursor = to;
                }
            }
            ClickState::SelectingLines {
                selected_line: (from, to),
            } => {
                if cursor < from {
                    let line_begin = self.cursor - self.find_line_begin(text);
                    let line_end = self.cursor + self.find_line_end(text);
                    self.selection = Some((line_begin, to));
                    self.cursor = line_end;
                } else if cursor > to {
                    let line_end = self.cursor + self.find_line_end(text);
                    self.selection = Some((from, line_end));
                    self.cursor = line_end;
                } else {
                    self.selection = Some((from, to));
                    self.cursor = to;
                }
            }
            _ => {}
        }

        self.last_click = cursor;
    }

    pub fn click_up(&mut self, _text: &Vec<char>) {
        self.click_state = ClickState::None;
        if let Some((from, to)) = self.selection {
            if from != to {
                self.click_state = ClickState::Selected;
            } else {
                self.selection = None;
            }
        }
    }

    pub fn undo(&mut self, text: &mut Vec<char>) {
        let command = self.undo_stack.pop();

        if let Some(command) = command {
            command.unapply(&mut self.cursor, text);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, text: &mut Vec<char>) {
        let command = self.redo_stack.pop();

        if let Some(command) = command {
            command.apply(&mut self.cursor, text);
            self.undo_stack.push(command);
        }
    }
}
