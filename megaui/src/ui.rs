use crate::{
    canvas::DrawCanvas, draw_command::CommandsList, draw_list::DrawList, types::Rect,
    types::Vector2, Id, InputHandler, Style,
};

use miniquad_text_rusttype::FontAtlas;
use std::{collections::HashMap, rc::Rc};

mod cursor;
mod input;
use cursor::Cursor;
use input::Input;

pub use cursor::Layout;
pub use input::{InputCharacter, Key, KeyCode};

pub(crate) struct Window {
    pub id: Id,
    pub parent: Option<Id>,
    pub visible: bool,
    // active is set to true when the begin_window is called on this window
    // and is going to be set to false at the end of each frame
    pub active: bool,
    // was the window "active" during the last frame
    // the way to find out wich windows should be rendered after end of the frame and during next frame, before begin_window of the next frame will be called on each window
    pub was_active: bool,
    pub title_height: f32,
    pub position: Vector2,
    pub size: Vector2,
    pub vertical_scroll_bar_width: f32,
    pub movable: bool,
    pub draw_commands: CommandsList,
    pub cursor: Cursor,
    pub childs: Vec<Id>,
    pub want_close: bool,
    pub input_focus: Option<Id>,
}

impl Window {
    pub fn new(
        id: Id,
        parent: Option<Id>,
        position: Vector2,
        size: Vector2,
        title_height: f32,
        margin: f32,
        movable: bool,
        font_atlas: Rc<FontAtlas>,
    ) -> Window {
        Window {
            id,
            position,
            size,
            vertical_scroll_bar_width: 0.,
            title_height,
            parent,
            visible: true,
            was_active: false,
            active: false,
            draw_commands: CommandsList::new(font_atlas),
            cursor: Cursor::new(
                Rect::new(
                    position.x,
                    position.y + title_height,
                    size.x,
                    size.y - title_height,
                ),
                margin,
            ),
            childs: vec![],
            want_close: false,
            movable,
            input_focus: None,
        }
    }

    pub fn input_focused(&self, id: Id) -> bool {
        self.input_focus
            .map_or(false, |input_focus| input_focus == id)
    }

    pub fn top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn full_rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn content_rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y + self.title_height,
            self.size.x - self.vertical_scroll_bar_width,
            self.size.y - self.title_height,
        )
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
        self.cursor.area.x = position.x;
        self.cursor.area.y = position.y + self.title_height;
    }

    pub fn title_rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.x,
            self.title_height,
        )
    }

    pub fn same_line(&mut self, x: f32) {
        self.cursor.next_same_line = Some(x);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DragState {
    Clicked(Vector2),
    Dragging(Vector2),
}

#[derive(Copy, Clone, Debug)]
pub enum Drag {
    No,
    Dragging(Vector2, Option<Id>),
    Dropped(Vector2, Option<Id>),
}

pub struct Ui {
    input: Input,
    pub(crate) style: Style,
    pub(crate) frame: u64,

    moving: Option<(Id, Vector2)>,
    windows: HashMap<Id, Window>,
    // special window that is always rendered on top of anything
    // TODO: maybe make modal windows stack instead
    modal: Option<Window>,
    windows_focus_order: Vec<Id>,

    storage_u32: HashMap<Id, u32>,
    storage_any: AnyStorage,

    dragging: Option<(Id, DragState)>,
    drag_hovered: Option<Id>,
    drag_hovered_previous_frame: Option<Id>,
    active_window: Option<Id>,
    child_window_stack: Vec<Id>,

    pub font_atlas: Rc<FontAtlas>,

    clipboard_selection: String,
    clipboard: Box<dyn crate::ClipboardObject>,
}

#[derive(Default)]
pub(crate) struct AnyStorage {
    storage: HashMap<Id, Box<dyn std::any::Any>>,
}

impl AnyStorage {
    pub(crate) fn get_or_insert_with<T: Default + 'static, F: Fn() -> T>(
        &mut self,
        id: Id,
        f: F,
    ) -> &mut T {
        self.storage
            .entry(id)
            .or_insert_with(|| Box::new(f()))
            .downcast_mut::<T>()
            .unwrap()
    }

    pub(crate) fn get_or_default<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.storage
            .entry(id)
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .unwrap()
    }
}
pub(crate) struct WindowContext<'a> {
    pub window: &'a mut Window,
    pub dragging: &'a mut Option<(Id, DragState)>,
    pub drag_hovered: &'a mut Option<Id>,
    pub drag_hovered_previous_frame: &'a mut Option<Id>,
    pub storage_u32: &'a mut HashMap<Id, u32>,
    pub storage_any: &'a mut AnyStorage,
    pub global_style: &'a Style,
    pub input: &'a mut Input,
    pub clipboard_selection: &'a mut String,
    pub clipboard: &'a mut dyn crate::ClipboardObject,
    pub focused: bool,
}

impl<'a> WindowContext<'a> {
    pub(crate) fn scroll_area(&mut self) {
        let inner_rect = self.window.cursor.scroll.inner_rect_previous_frame;
        let rect = self.window.content_rect();
        let rect = Rect {
            w: rect.w + self.window.vertical_scroll_bar_width,
            ..rect
        };

        self.window.cursor.scroll.scroll = Vector2::new(
            -self.window.cursor.scroll.rect.x,
            -self.window.cursor.scroll.rect.y,
        );

        if inner_rect.h > rect.h {
            self.window.vertical_scroll_bar_width = self.global_style.scroll_width;
            self.draw_vertical_scroll_bar(
                rect,
                Rect::new(
                    rect.x + rect.w - self.global_style.scroll_width,
                    rect.y,
                    self.global_style.scroll_width,
                    rect.h,
                ),
            );
        } else {
            self.window.vertical_scroll_bar_width = 0.;
        }

        self.window.cursor.scroll.update();
    }

    pub(crate) fn close(&mut self) {
        self.window.want_close = true;
    }

    fn draw_vertical_scroll_bar(&mut self, area: Rect, rect: Rect) {
        let mut scroll = &mut self.window.cursor.scroll;
        let inner_rect = scroll.inner_rect_previous_frame;
        let size = scroll.rect.h / inner_rect.h * rect.h;
        let pos = (scroll.rect.y - inner_rect.y) / inner_rect.h * rect.h;

        self.window.draw_commands.draw_line(
            Vector2::new(rect.x, rect.y),
            Vector2::new(rect.x, rect.y + rect.h),
            self.global_style.window_border(self.focused),
        );

        let mut clicked = false;
        let mut hovered = false;
        let bar = Rect::new(rect.x + 1., rect.y + pos, rect.w - 1., size);
        let k = inner_rect.h / scroll.rect.h;
        if bar.contains(self.input.mouse_position) {
            hovered = true;
        }
        if hovered && self.input.is_mouse_down() {
            self.input.cursor_grabbed = true;
            scroll.dragging_y = true;
            scroll.initial_scroll.y = scroll.rect.y - self.input.mouse_position.y * k;
        }
        if scroll.dragging_y && self.input.is_mouse_down == false {
            self.input.cursor_grabbed = false;
            scroll.dragging_y = false;
        }
        if scroll.dragging_y {
            clicked = true;
            scroll.scroll_to(self.input.mouse_position.y * k + scroll.initial_scroll.y);
        }

        if self.focused
            && area.contains(self.input.mouse_position)
            && self.input.mouse_wheel.y != 0.
        {
            scroll.scroll_to(
                scroll.rect.y + self.input.mouse_wheel.y * k * self.global_style.scroll_multiplier,
            );
        }

        self.window.draw_commands.draw_rect(
            bar,
            None,
            self.global_style
                .scroll_bar_handle(self.focused, hovered, clicked),
        );
    }
}

impl InputHandler for Ui {
    fn mouse_down(&mut self, position: (f32, f32)) {
        let position = Vector2::new(position.0, position.1);

        self.input.is_mouse_down = true;
        self.input.click_down = true;
        self.input.mouse_position = position;

        if let Some(ref window) = self.modal {
            let rect = Rect::new(
                window.position.x,
                window.position.y,
                window.size.x,
                window.size.y,
            );
            if window.was_active && rect.contains(position) {
                return;
            }
        }

        for (n, window) in self.windows_focus_order.iter().enumerate() {
            let window = &self.windows[window];

            if window.was_active == false {
                continue;
            }

            if window.top_level() && window.title_rect().contains(position) && window.movable {
                self.moving = Some((
                    window.id,
                    position - Vector2::new(window.position.x, window.position.y),
                ));
            }

            if window.top_level() && window.full_rect().contains(position) {
                let window = self.windows_focus_order.remove(n);
                self.windows_focus_order.insert(0, window);
                return;
            }
        }
    }

    fn mouse_up(&mut self, _: (f32, f32)) {
        self.input.is_mouse_down = false;
        self.input.click_up = true;
        self.moving = None;
    }

    fn mouse_wheel(&mut self, x: f32, y: f32) {
        self.input.mouse_wheel = Vector2::new(x, y);
    }

    fn mouse_move(&mut self, position: (f32, f32)) {
        let position = Vector2::new(position.0, position.1);

        self.input.mouse_position = position;
        if let Some((id, orig)) = self.moving.as_ref() {
            self.windows
                .get_mut(id)
                .unwrap()
                .set_position(Vector2::new(position.x - orig.x, position.y - orig.y));
        }
    }

    fn char_event(&mut self, character: char, shift: bool, ctrl: bool) {
        self.input.input_buffer.push(input::InputCharacter {
            key: input::Key::Char(character),
            modifier_shift: shift,
            modifier_ctrl: ctrl,
        });
    }

    fn key_down(&mut self, key: crate::input_handler::KeyCode, shift: bool, ctrl: bool) {
        if ctrl && (key == KeyCode::C || key == KeyCode::X) {
            self.clipboard.set(&self.clipboard_selection);
        }
        self.input.input_buffer.push(input::InputCharacter {
            key: input::Key::KeyCode(key),
            modifier_shift: shift,
            modifier_ctrl: ctrl,
        });
    }
}

impl Ui {
    pub fn new() -> Ui {
        let mut font_atlas = FontAtlas::new(
            &include_bytes!("../assets/ProggyClean.ttf")[..],
            13,
            FontAtlas::ascii_character_list(),
        )
        .unwrap();

        let white_square = [(0, 0), (1, 0), (1, 1), (0, 1)];
        let w = font_atlas.texture.width;
        for pixel in white_square.iter() {
            font_atlas.texture.data[(pixel.0 + w * pixel.1 + 0) as usize] = 255;
            font_atlas.texture.data[(pixel.0 + w * pixel.1 + 1) as usize] = 255;
            font_atlas.texture.data[(pixel.0 + w * pixel.1 + 2) as usize] = 255;
            font_atlas.texture.data[(pixel.0 + w * pixel.1 + 3) as usize] = 255;
        }

        Ui {
            input: Input::default(),
            style: Style::default(),
            frame: 0,
            moving: None,
            windows: HashMap::default(),
            modal: None,
            windows_focus_order: vec![],
            dragging: None,
            active_window: None,
            child_window_stack: vec![],
            drag_hovered: None,
            drag_hovered_previous_frame: None,
            storage_u32: HashMap::default(),
            storage_any: AnyStorage::default(),
            font_atlas: Rc::new(font_atlas),
            clipboard_selection: String::new(),
            clipboard: Box::new(crate::LocalClipboard::new()),
        }
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub(crate) fn begin_window(
        &mut self,
        id: Id,
        parent: Option<Id>,
        position: Vector2,
        size: Vector2,
        title_height: f32,
        movable: bool,
    ) -> WindowContext {
        if let Some(active_window) = self.active_window {
            self.child_window_stack.push(active_window);
        }
        self.active_window = Some(id);

        let focused = self.is_focused(id);
        let margin = self.style.margin;
        let font_atlas = self.font_atlas.clone();
        let windows_focus_order = &mut self.windows_focus_order;

        let parent_clip_rect = if let Some(parent) = parent {
            self.windows
                .get(&parent)
                .and_then(|window| window.draw_commands.clipping_zone)
        } else {
            None
        };

        let window = &mut *self.windows.entry(id).or_insert_with(|| {
            if parent.is_none() {
                windows_focus_order.push(id);
            }

            Window::new(
                id,
                parent,
                position,
                size,
                title_height,
                margin,
                movable,
                font_atlas,
            )
        });

        window.size = size;
        window.want_close = false;
        window.active = true;
        window.draw_commands.clipping_zone = parent_clip_rect;

        // top level windows are moveble, so we update their position only on the first frame
        // while the child windows are not moveble and should update their position each frame
        if parent.is_some() {
            window.set_position(position);
        }

        WindowContext {
            focused,
            window,
            input: &mut self.input,
            global_style: &self.style,
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
        }
    }

    pub(crate) fn begin_modal(
        &mut self,
        id: Id,
        position: Vector2,
        size: Vector2,
    ) -> WindowContext {
        let font_atlas = self.font_atlas.clone();

        let window = self.modal.get_or_insert_with(|| {
            Window::new(id, None, position, size, 0.0, 0.0, false, font_atlas)
        });

        window.size = size;
        window.want_close = false;
        window.active = true;
        window.draw_commands.clipping_zone =
            Some(Rect::new(position.x, position.y, size.x, size.y));
        window.set_position(position);

        WindowContext {
            focused: true,
            window,
            input: &mut self.input,
            global_style: &self.style,
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
        }
    }

    pub(crate) fn end_modal(&mut self) {
        // do nothing
        // modal window will end itself while its the only modal window
    }

    pub(crate) fn end_window(&mut self) {
        self.active_window = self.child_window_stack.pop();
    }

    pub(crate) fn get_active_window_context(&mut self) -> WindowContext {
        let active_window = self
            .active_window
            .expect("Rendering outside of window unsupported");
        let focused = self.is_focused(active_window);
        let window = self.windows.get_mut(&active_window).unwrap();

        WindowContext {
            window,
            focused,
            input: &mut self.input,
            global_style: &self.style,
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
        }
    }

    pub fn set_clipboard_object<T: crate::ClipboardObject + 'static>(&mut self, clipboard: T) {
        self.clipboard = Box::new(clipboard);
    }

    pub fn is_mouse_over(&self, mouse_position: Vector2) -> bool {
        for window in self.windows_focus_order.iter() {
            let window = &self.windows[window];
            if window.was_active == false {
                continue;
            }
            if window.full_rect().contains(mouse_position) {
                return true;
            }
        }
        false
    }

    pub fn active_window_focused(&self) -> bool {
        self.active_window.map_or(false, |wnd| self.is_focused(wnd))
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    pub fn close_current_window(&mut self) {
        let mut context = self.get_active_window_context();
        context.close();
    }

    pub fn is_focused(&self, id: Id) -> bool {
        if let Some(focused_window) = self
            .windows_focus_order
            .iter()
            .find(|window| self.windows[window].was_active || self.windows[window].active)
        {
            if id == *focused_window {
                return true;
            }
            if let Some(parent) = self.child_window_stack.get(0) {
                return *parent == *focused_window;
            }
        }
        return false;
    }

    pub fn new_frame(&mut self) {
        self.frame += 1;

        self.drag_hovered_previous_frame = self.drag_hovered;
        self.drag_hovered = None;
        self.input.reset();

        for (_, window) in &mut self.windows {
            window.draw_commands.clear();
            window.cursor.reset();
            window.was_active = window.active;
            window.active = false;
            window.childs.clear();
        }

        for window in &mut self.modal {
            window.draw_commands.clear();
            window.cursor.reset();
            window.was_active = window.active;
            window.active = false;
            window.childs.clear();

            self.input.modal_active = window.was_active;
        }
    }

    pub fn render(&mut self, draw_list: &mut Vec<DrawList>) {
        for window in self.windows_focus_order.iter().rev() {
            let window = &self.windows[window];
            if window.was_active {
                self.render_window(window, Vector2::new(0., 0.), draw_list);
            }
        }
        if let Some(modal) = self.modal.as_ref() {
            if modal.was_active {
                self.render_window(modal, Vector2::new(0., 0.), draw_list);
            }
        }

        if let Some((id, DragState::Dragging(orig))) = self.dragging {
            let window = &self.windows[&id];

            self.render_window(window, self.input.mouse_position - orig, draw_list);
        }
    }

    fn render_window(&self, window: &Window, offset: Vector2, draw_list: &mut Vec<DrawList>) {
        for cmd in &window.draw_commands.commands {
            crate::draw_list::render_command(draw_list, cmd.offset(offset));
        }

        for child in &window.childs {
            let child_window = &self.windows[child];
            if window.content_rect().overlaps(&child_window.full_rect()) {
                self.render_window(child_window, offset, draw_list);
            }
        }
    }

    pub fn focus_window(&mut self, id: Id) {
        if let Some(n) = self.windows_focus_order.iter().position(|win| *win == id) {
            let window = self.windows_focus_order.remove(n);
            self.windows_focus_order.insert(0, window);
        }
    }

    pub fn move_window(&mut self, id: Id, position: Vector2) {
        if let Some(window) = self.windows.get_mut(&id) {
            window.set_position(position);
        }
    }

    pub fn same_line(&mut self, x: f32) {
        let context = self.get_active_window_context();
        context.window.same_line(x);
    }

    pub fn canvas(&mut self) -> DrawCanvas {
        let context = self.get_active_window_context();

        DrawCanvas { context }
    }
}
