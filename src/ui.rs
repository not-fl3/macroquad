//! Immediate mode UI.
//!
//! Spiritual successor of megaui library, but fully skinnable and configurable.
//!
//! The UI entrypoint is `root_ui()` call.
//! ```ignore
//! root_ui().label(None, "hello megaui");
//! if root_ui().button(None, "Push me") {
//!    println!("pushed");
//! }
//! ```
//! This will draw a label and a button one after each other right on top of the
//! screen.

pub mod canvas;
mod clipboard;
#[macro_use]
mod hash;
mod input_handler;
mod render;
mod style;

pub mod widgets;

pub use clipboard::ClipboardObject;
pub use input_handler::{InputHandler, KeyCode};
pub use render::{DrawList, Vertex};
pub use style::{Skin, Style, StyleBuilder};

pub use crate::hash;

pub(crate) use render::ElementState;

use std::{
    borrow::Cow,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

/// Root UI. Widgets drawn with the root ui will be always presented at the end of the frame with a "default" camera.
/// UI space would be a "default" screen space (0..screen_width(), 0..screen_height())
pub fn root_ui() -> impl DerefMut<Target = Ui> {
    crate::get_context().ui_context.ui.borrow_mut()
}

/// Current camera world space UI.
/// Widgets will be drawn either at the end of the frame or just before next "set_camera" clal
/// UI space would be equal to the camera space, widgets will be drawn at the plane with Y up X right and Z = 0.
/// Note that windows focus queue, input focus etc is shared across all cameras.
/// So this:
///
/// ```skip
/// camera_ui().draw_window();
/// set_camera(..);
/// camera_ui().draw_window();
/// root_ui().draw_window();
/// ```
/// Will result 3 windows on the screen, all in different cameras and probably looking differently,
/// but only one of them would be focused.
#[doc(hidden)]
#[allow(unreachable_code)]
pub fn camera_ui() -> impl DerefMut<Target = Ui> {
    unimplemented!() as &'static mut Ui
}

use crate::{
    math::{Rect, RectOffset, Vec2},
    text::{
        atlas::{Atlas, SpriteKey},
        Font,
    },
    texture::Image,
    ui::{canvas::DrawCanvas, render::Painter},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod cursor;
mod input;
mod key_repeat;

use cursor::Cursor;
use input::Input;

pub use cursor::Layout;
use input::{InputCharacter, Key};

/// Is used to keep track of internal state of various widgets like [widgets::Window](macroquad::ui::widgets::Window)
/// These should be unique per window and ideally not change in between frames.
pub type Id = u64;

pub enum UiContent<'a> {
    Label(Cow<'a, str>),
    Texture(crate::texture::Texture2D),
}

impl<'a> From<&'a str> for UiContent<'a> {
    fn from(data: &'a str) -> UiContent<'a> {
        UiContent::Label(data.into())
    }
}

impl From<String> for UiContent<'static> {
    fn from(data: String) -> UiContent<'static> {
        UiContent::Label(data.into())
    }
}

impl From<crate::texture::Texture2D> for UiContent<'static> {
    fn from(data: crate::texture::Texture2D) -> UiContent<'static> {
        UiContent::Texture(data)
    }
}

pub(crate) struct Window {
    pub id: Id,
    pub parent: Option<Id>,
    // active is set to true when the begin_window is called on this window
    // and is going to be set to false at the end of each frame
    pub active: bool,
    // was the window "active" during the last frame
    // the way to find out which windows should be rendered after end of the frame and during next frame, before begin_window of the next frame will be called on each window
    pub was_active: bool,
    pub title_height: f32,
    pub position: Vec2,
    pub size: Vec2,
    pub vertical_scroll_bar_width: f32,
    pub movable: bool,
    pub painter: Painter,
    pub cursor: Cursor,
    pub childs: Vec<Id>,
    pub want_close: bool,
    pub force_focus: bool,

    margin: f32,
    window_margin: RectOffset,
}

impl Window {
    pub fn new(
        id: Id,
        parent: Option<Id>,
        position: Vec2,
        size: Vec2,
        title_height: f32,
        window_margin: RectOffset,
        margin: f32,
        movable: bool,
        force_focus: bool,
        atlas: Arc<Mutex<Atlas>>,
    ) -> Window {
        Window {
            id,
            position,
            size,
            vertical_scroll_bar_width: 0.,
            title_height,
            parent,
            was_active: false,
            active: false,
            painter: Painter::new(atlas),
            cursor: Cursor::new(
                Rect::new(
                    position.x + window_margin.left,
                    position.y + title_height + window_margin.top,
                    size.x - window_margin.left - window_margin.right,
                    size.y - title_height - window_margin.top - window_margin.bottom,
                ),
                margin,
            ),
            margin,
            window_margin,
            childs: vec![],
            want_close: false,
            movable,
            force_focus,
        }
    }

    pub fn resize(&mut self, size: Vec2) {
        self.size = size;
        self.cursor = Cursor::new(
            Rect::new(
                self.position.x + self.window_margin.left,
                self.position.y + self.title_height + self.window_margin.top,
                self.size.x - self.window_margin.left - self.window_margin.right,
                self.size.y
                    - self.title_height
                    - self.window_margin.top
                    - self.window_margin.bottom,
            ),
            self.margin,
        );
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

    pub fn set_position(&mut self, position: Vec2) {
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
    Clicked(Vec2),
    Dragging(Vec2),
}

#[derive(Copy, Clone, Debug)]
pub enum Drag {
    No,
    Dragging(Vec2, Option<Id>),
    Dropped(Vec2, Option<Id>),
}

struct StyleStack {
    default_skin: Skin,
    custom_skin_stack: Vec<Skin>,
}

impl StyleStack {
    fn new(atlas: Arc<Mutex<Atlas>>, default_font: Arc<Mutex<Font>>) -> StyleStack {
        StyleStack {
            default_skin: Skin::new(atlas, default_font),
            custom_skin_stack: vec![],
        }
    }

    fn top(&self) -> &Skin {
        self.custom_skin_stack.last().unwrap_or(&self.default_skin)
    }
}

pub(crate) struct TabSelector {
    counter: isize,
    wants: Option<isize>,
    to_change: Option<isize>,
}

impl TabSelector {
    fn new() -> Self {
        TabSelector {
            counter: 0,
            wants: None,
            to_change: None,
        }
    }

    fn new_frame(&mut self) {
        self.to_change = if self.wants == Some(-1) {
            Some(self.counter - 1)
        } else if self.wants == Some(self.counter) {
            Some(0)
        } else {
            self.wants
        };
        self.wants = None;
        self.counter = 0;
    }

    /// Returns true if this widget should gain focus, because user pressed `Tab` or `Shift + Tab`.
    pub(crate) fn register_selectable_widget(&mut self, has_focus: bool, input: &Input) -> bool {
        if has_focus {
            enum PressedTabKey {
                Tab,
                ShiftTab,
                Other,
            }

            let key = if input
                .input_buffer
                .iter()
                .any(|inp| inp.key == Key::KeyCode(KeyCode::Tab) && inp.modifier_shift)
            {
                PressedTabKey::ShiftTab
            } else if input
                .input_buffer
                .iter()
                .any(|inp| inp.key == Key::KeyCode(KeyCode::Tab))
            {
                PressedTabKey::Tab
            } else {
                PressedTabKey::Other
            };

            match key {
                PressedTabKey::Tab => self.wants = Some(self.counter + 1),
                PressedTabKey::ShiftTab => self.wants = Some(self.counter - 1),
                PressedTabKey::Other => {}
            }
        }

        let result = if self.to_change.map(|id| id == self.counter).unwrap_or(false) {
            self.to_change = None;
            true
        } else {
            false
        };

        self.counter += 1;

        result
    }
}

pub struct Ui {
    input: Input,
    skin_stack: StyleStack,
    /// Returns the number of frames that have elapsed since the program started.
    pub frame: u64,
    pub(crate) time: f32,

    moving: Option<(Id, Vec2)>,
    windows: HashMap<Id, Window>,
    // special window that is always rendered on top of anything
    // TODO: maybe make modal windows stack instead
    modal: Option<Window>,
    // another special window
    // always rendered behind everything and do not have borders or scrolls
    // helps using window-less uis
    root_window: Window,
    windows_focus_order: Vec<Id>,

    storage_u32: HashMap<Id, u32>,
    storage_any: AnyStorage,

    dragging: Option<(Id, DragState)>,
    drag_hovered: Option<Id>,
    drag_hovered_previous_frame: Option<Id>,
    active_window: Option<Id>,
    hovered_window: Id,
    in_modal: bool,
    child_window_stack: Vec<Id>,

    last_item_clicked: bool,
    last_item_hovered: bool,

    pub(crate) atlas: Arc<Mutex<Atlas>>,
    pub(crate) default_font: Arc<Mutex<Font>>,

    clipboard_selection: String,
    clipboard: Box<dyn crate::ui::ClipboardObject>,

    key_repeat: key_repeat::KeyRepeat,

    tab_selector: TabSelector,
    input_focus: Option<Id>,
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
    pub style: &'a Skin,
    pub input: &'a mut Input,
    pub clipboard_selection: &'a mut String,
    pub clipboard: &'a mut dyn crate::ui::ClipboardObject,
    pub focused: bool,
    pub last_item_clicked: &'a mut bool,
    pub last_item_hovered: &'a mut bool,
    pub tab_selector: &'a mut TabSelector,
    pub input_focus: &'a mut Option<Id>,
}

impl<'a> WindowContext<'a> {
    pub(crate) fn scroll_area(&mut self) {
        let inner_rect = self.window.cursor.scroll.inner_rect_previous_frame;
        let rect = self.window.content_rect();
        let rect = Rect {
            w: rect.w + self.window.vertical_scroll_bar_width,
            ..rect
        };

        self.window.cursor.scroll.scroll = Vec2::new(
            -self.window.cursor.scroll.rect.x,
            -self.window.cursor.scroll.rect.y,
        );

        if inner_rect.h > rect.h {
            self.window.vertical_scroll_bar_width = self.style.scroll_width;
            self.draw_vertical_scroll_bar(
                rect,
                Rect::new(
                    rect.x + rect.w - self.style.scroll_width,
                    rect.y,
                    self.style.scroll_width,
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

        self.window.painter.draw_line(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x, rect.y + rect.h),
            self.style.scrollbar_style.color(ElementState {
                focused: self.focused,
                ..Default::default()
            }),
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
                scroll.rect.y + self.input.mouse_wheel.y * k * self.style.scroll_multiplier,
            );
        }

        self.window.painter.draw_rect(
            bar,
            None,
            self.style.scrollbar_handle_style.color(ElementState {
                focused: self.focused,
                hovered,
                clicked,
                selected: false,
            }),
        );
    }

    pub fn register_click_intention(&mut self, rect: Rect) -> (bool, bool) {
        *self.last_item_hovered =
            self.input.window_active && rect.contains(self.input.mouse_position);
        *self.last_item_clicked = *self.last_item_hovered && self.input.click_down();

        (*self.last_item_hovered, *self.last_item_clicked)
    }

    pub fn input_focused(&self, id: Id) -> bool {
        self.input_focus
            .map_or(false, |input_focus| input_focus == id)
    }
}

impl InputHandler for Ui {
    fn mouse_down(&mut self, position: (f32, f32)) {
        let position = Vec2::new(position.0, position.1);

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
                    position - Vec2::new(window.position.x, window.position.y),
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
        self.input.mouse_wheel = Vec2::new(x, y);
    }

    fn mouse_move(&mut self, position: (f32, f32)) {
        let position = Vec2::new(position.0, position.1);

        // assuming that the click was to the root window
        // if it is not - hovered_window will be set a little later in that function
        self.hovered_window = 0;
        for window in self.windows_focus_order.iter() {
            let window = &self.windows[window];

            if window.top_level() && window.was_active && window.full_rect().contains(position) {
                self.hovered_window = window.id;
                break;
            }
        }

        match &self.modal {
            Some(modal) if modal.was_active || modal.active => {
                if modal.full_rect().contains(position) {
                    self.hovered_window = modal.id;
                }
            }
            _ => {}
        }

        self.input.mouse_position = position;
        if let Some((id, orig)) = self.moving.as_ref() {
            self.windows
                .get_mut(id)
                .unwrap()
                .set_position(Vec2::new(position.x - orig.x, position.y - orig.y));
        }
    }

    fn char_event(&mut self, character: char, shift: bool, ctrl: bool) {
        self.input.modifier_ctrl = ctrl;
        self.input.input_buffer.push(input::InputCharacter {
            key: input::Key::Char(character),
            modifier_shift: shift,
            modifier_ctrl: ctrl,
        });
    }

    fn key_down(&mut self, key: KeyCode, shift: bool, ctrl: bool) {
        self.input.modifier_ctrl = ctrl;

        if key == KeyCode::Escape {
            self.input.escape = true;
        }
        if key == KeyCode::Enter {
            self.input.enter = true;
        }

        if ctrl && (key == KeyCode::C || key == KeyCode::X) {
            self.clipboard.set(&self.clipboard_selection);
        }

        if key != KeyCode::Control && self.key_repeat.add_repeat_gap(key, self.time) {
            self.input.input_buffer.push(input::InputCharacter {
                key: input::Key::KeyCode(key),
                modifier_shift: shift,
                modifier_ctrl: ctrl,
            });
        }
    }
}

impl Ui {
    pub fn new(
        ctx: &mut dyn miniquad::RenderingBackend,
        screen_width: f32,
        screen_height: f32,
    ) -> Ui {
        let atlas = Arc::new(Mutex::new(Atlas::new(ctx, miniquad::FilterMode::Nearest)));
        let mut font =
            crate::text::Font::load_from_bytes(atlas.clone(), include_bytes!("ProggyClean.ttf"))
                .unwrap();

        for character in crate::text::Font::ascii_character_list() {
            font.cache_glyph(character, 13);
        }

        atlas
            .lock()
            .unwrap()
            .cache_sprite(SpriteKey::Id(0), Image::gen_image_color(1, 1, crate::WHITE));

        let font = Arc::new(Mutex::new(font));
        Ui {
            input: Input::default(),
            default_font: font.clone(),
            skin_stack: StyleStack::new(atlas.clone(), font),
            frame: 0,
            moving: None,
            windows: HashMap::default(),
            modal: None,
            root_window: {
                let mut window = Window::new(
                    0,
                    None,
                    Vec2::new(0., 0.),
                    Vec2::new(screen_width, screen_height),
                    0.0,
                    RectOffset::new(0.0, 0.0, 0.0, 0.0),
                    0.0,
                    false,
                    true,
                    atlas.clone(),
                );
                window.active = true;
                window.was_active = true;
                window
            },
            windows_focus_order: vec![],
            dragging: None,
            active_window: None,
            hovered_window: 0,
            in_modal: false,
            child_window_stack: vec![],
            drag_hovered: None,
            drag_hovered_previous_frame: None,
            storage_u32: HashMap::default(),
            storage_any: AnyStorage::default(),
            atlas,
            clipboard_selection: String::new(),
            clipboard: Box::new(ui_context::ClipboardObject),
            time: 0.0,
            key_repeat: key_repeat::KeyRepeat::new(),
            last_item_clicked: false,
            last_item_hovered: false,
            tab_selector: TabSelector::new(),
            input_focus: None,
        }
    }

    pub fn set_default_skin(&mut self, _skin: Skin) {
        unimplemented!()
    }

    pub fn style_builder(&self) -> StyleBuilder {
        StyleBuilder::new(self.default_font.clone(), self.atlas.clone())
    }

    pub fn default_skin(&self) -> Skin {
        self.skin_stack.top().clone()
    }

    pub(crate) fn begin_window(
        &mut self,
        id: Id,
        parent: Option<Id>,
        position: Vec2,
        size: Vec2,
        titlebar: bool,
        movable: bool,
    ) -> WindowContext {
        if parent.is_some() {
            self.child_window_stack
                .push(self.active_window.unwrap_or(0));
        }
        self.input.window_active = self.is_input_hovered(id);

        self.active_window = Some(id);

        let focused = self.is_focused(id);
        let margin = self.skin_stack.top().margin;
        let margin_window = self.skin_stack.top().window_style.border_margin();

        let title_height = if titlebar {
            self.skin_stack.top().title_height
        } else {
            0.
        };
        let atlas = self.atlas.clone();
        let windows_focus_order = &mut self.windows_focus_order;

        let parent_force_focus = match parent {
            // childs of root window are always force_focused
            Some(0) => true,
            // childs of force_focused windows are always force_focused as well
            Some(parent) => self
                .windows
                .get(&parent)
                .map_or(false, |window| window.force_focus),
            _ => false,
        };
        let parent_clip_rect = if let Some(parent) = parent {
            self.windows
                .get(&parent)
                .and_then(|window| window.painter.clipping_zone)
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
                margin_window,
                margin,
                movable,
                parent_force_focus,
                atlas,
            )
        });
        if !window.movable {
            window.set_position(position);
        }
        window.size = size;
        window.want_close = false;
        window.active = true;
        window.painter.clipping_zone = parent_clip_rect;

        // top level windows are movable, so we update their position only on the first frame
        // while the child windows are not movable and should update their position each frame
        if parent.is_some() {
            window.set_position(position);
        }

        WindowContext {
            focused,
            window,
            input: &mut self.input,
            style: self.skin_stack.top(),
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
            last_item_clicked: &mut self.last_item_clicked,
            last_item_hovered: &mut self.last_item_hovered,
            tab_selector: &mut self.tab_selector,
            input_focus: &mut self.input_focus,
        }
    }

    pub(crate) fn begin_modal(&mut self, id: Id, position: Vec2, size: Vec2) -> WindowContext {
        self.input.window_active = true;
        self.in_modal = true;

        let atlas = self.atlas.clone();

        let window = self.modal.get_or_insert_with(|| {
            Window::new(
                id,
                None,
                position,
                size,
                0.0,
                RectOffset::new(0.0, 0.0, 0.0, 0.0),
                0.0,
                false,
                true,
                atlas,
            )
        });

        window.parent = self.active_window;
        window.size = size;
        window.want_close = false;
        window.active = true;
        window.painter.clipping_zone = Some(Rect::new(position.x, position.y, size.x, size.y));
        window.set_position(position);

        WindowContext {
            focused: true,
            window,
            input: &mut self.input,
            style: self.skin_stack.top(),
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
            last_item_clicked: &mut self.last_item_clicked,
            last_item_hovered: &mut self.last_item_hovered,
            tab_selector: &mut self.tab_selector,
            input_focus: &mut self.input_focus,
        }
    }

    pub(crate) fn end_modal(&mut self) {
        self.in_modal = false;
        self.input.window_active = self.is_input_hovered(self.active_window.unwrap_or(0));
    }

    pub(crate) fn end_window(&mut self) {
        self.active_window = self.child_window_stack.pop();
        self.input.window_active = self.is_input_hovered(self.active_window.unwrap_or(0));
    }

    pub(crate) fn get_active_window_context(&mut self) -> WindowContext {
        let focused;
        let window = if self.in_modal == false {
            match self.active_window {
                None | Some(0) => {
                    focused = true;
                    &mut self.root_window
                }
                Some(active_window) => {
                    focused = self.is_focused(active_window);
                    self.windows.get_mut(&active_window).unwrap()
                }
            }
        } else {
            focused = true;
            self.modal.as_mut().unwrap()
        };

        WindowContext {
            window,
            focused,
            input: &mut self.input,
            style: self.skin_stack.top(),
            dragging: &mut self.dragging,
            drag_hovered: &mut self.drag_hovered,
            drag_hovered_previous_frame: &mut self.drag_hovered_previous_frame,
            storage_u32: &mut self.storage_u32,
            storage_any: &mut self.storage_any,
            clipboard_selection: &mut self.clipboard_selection,
            clipboard: &mut *self.clipboard,
            last_item_clicked: &mut self.last_item_clicked,
            last_item_hovered: &mut self.last_item_hovered,
            tab_selector: &mut self.tab_selector,
            input_focus: &mut self.input_focus,
        }
    }

    /// Returns true if the last widget which had `.ui` called on it is being clicked.
    pub fn last_item_clicked(&mut self) -> bool {
        self.last_item_clicked
    }

    /// Returns true if the mouse is over the last widget which had `.ui` called on it.
    pub fn last_item_hovered(&mut self) -> bool {
        self.last_item_hovered
    }

    /// Scrolls the middle of the active GUI window to its GUI cursor
    ///
    /// Note that this does not work on the first frame of the GUI application.
    /// If you want your widget to start with its scrollbar in a particular location,
    /// consider `if ui.frame == 1 { ui.scroll_here() }`.
    pub fn scroll_here(&mut self) {
        self.scroll_here_ratio(0.5)
    }

    /// Scrolls the active GUI window to its GUI cursor.
    ///
    /// 1.0 puts the bottom of the window at the GUI cursor,
    /// 0.0 puts the top of the window there.
    ///
    /// 0.5 as the ratio puts the middle of the window at the GUI cursor,
    /// and is equivalent to `Ui::scroll_here`.
    pub fn scroll_here_ratio(&mut self, ratio: f32) {
        let context = self.get_active_window_context();
        let cursor = &mut context.window.cursor;
        cursor.scroll.scroll_to(cursor.y - cursor.area.h * ratio);
    }

    /// How far the active gui window has been scrolled down on the y axis.
    ///
    /// Note that for these purposes, a Group widget is still considered a Window
    /// because it can have its own scrollbar.
    pub fn scroll(&mut self) -> Vec2 {
        self.get_active_window_context().window.cursor.scroll.scroll
    }

    /// The farthest down a scrollbar may go given the constraints of its window.
    ///
    /// Note that for these purposes, a Group widget is still considered a Window
    /// because it can have its own scrollbar.
    pub fn scroll_max(&mut self) -> Vec2 {
        let cursor = &self.get_active_window_context().window.cursor;
        Vec2::new(
            cursor.scroll.inner_rect.w - cursor.area.w,
            cursor.scroll.inner_rect.h - cursor.area.h,
        )
    }

    pub fn is_mouse_captured(&self) -> bool {
        self.input.cursor_grabbed
    }

    pub fn is_mouse_over(&self, mouse_position: Vec2) -> bool {
        for window in self.windows_focus_order.iter() {
            let window = &self.windows[window];
            if window.was_active == false {
                continue;
            }
            if window.full_rect().contains(mouse_position) {
                return true;
            }
        }
        for window in &self.modal {
            if window.was_active {
                if window.full_rect().contains(mouse_position) {
                    return true;
                }
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

    fn is_input_hovered(&self, id: Id) -> bool {
        // if thats exactly the clicked window - it's always the hovered one
        if id == self.hovered_window {
            return true;
        }

        // hovered window is always the root window and the given id may be the child
        // window id
        // so need to figure the root id

        if self.in_modal {
            true
        } else {
            self.child_window_stack
                .get(0)
                .map_or(false, |root| *root == self.hovered_window)
        }
    }

    fn is_focused(&self, id: Id) -> bool {
        if self
            .windows
            .get(&id)
            .map_or(false, |window| window.force_focus)
        {
            return true;
        }

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

        false
    }

    pub fn new_frame(&mut self, delta: f32) {
        self.root_window.resize(crate::math::vec2(
            crate::window::screen_width(),
            crate::window::screen_height(),
        ));

        self.frame += 1;
        self.time += delta;

        self.last_item_clicked = false;
        self.last_item_hovered = false;

        self.drag_hovered_previous_frame = self.drag_hovered;
        self.drag_hovered = None;
        self.input.reset();
        self.input.window_active = self.hovered_window == 0;

        self.tab_selector.new_frame();

        self.key_repeat.new_frame(self.time);

        for (_, window) in &mut self.windows {
            window.painter.clear();
            window.cursor.reset();
            window.was_active = window.active;
            window.active = false;
            window.childs.clear();
        }

        for window in &mut self.modal {
            window.painter.clear();
            window.cursor.reset();
            window.was_active = window.active;
            window.active = false;
            window.childs.clear();
        }

        {
            self.root_window.painter.clear();
            self.root_window.cursor.reset();
            self.root_window.childs.clear();
        }
    }

    pub fn render(&mut self, draw_list: &mut Vec<DrawList>) {
        self.render_window(&self.root_window, Vec2::new(0., 0.), draw_list);

        for window in self.windows_focus_order.iter().rev() {
            let window = &self.windows[window];
            if window.was_active {
                self.render_window(window, Vec2::new(0., 0.), draw_list);
            }
        }

        if let Some(modal) = self.modal.as_ref() {
            if modal.was_active {
                self.render_window(modal, Vec2::new(0., 0.), draw_list);
            }
        }

        if let Some((id, DragState::Dragging(orig))) = self.dragging {
            let window = &self.windows[&id];

            self.render_window(window, self.input.mouse_position - orig, draw_list);
        }
    }

    fn render_window(&self, window: &Window, offset: Vec2, draw_list: &mut Vec<DrawList>) {
        for cmd in &window.painter.commands {
            crate::ui::render::render_command(draw_list, cmd.offset(offset));
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

    pub fn set_input_focus(&mut self, id: Id) {
        self.input_focus = Some(id);
    }

    pub fn clear_input_focus(&mut self) {
        self.input_focus = None;
    }

    pub fn move_window(&mut self, id: Id, position: Vec2) {
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

    /// small hack to keep some internal state
    /// used like this:
    /// ```skip
    /// if ui.last_item_clicked() {
    ///     *ui.get_bool(hash!("color picker opened")) ^= true;
    /// }
    /// if *ui.get_bool(hash!("color picker opened"))  {
    /// }
    /// ```
    pub fn get_bool(&mut self, id: Id) -> &mut bool {
        self.storage_any.get_or_default(id)
    }

    pub fn get_any<T: std::any::Any + Default>(&mut self, id: Id) -> &mut T {
        self.storage_any.get_or_default(id)
    }

    pub fn push_skin(&mut self, skin: &Skin) {
        self.skin_stack.custom_skin_stack.push(skin.clone());
    }

    pub fn pop_skin(&mut self) {
        self.skin_stack.custom_skin_stack.pop();
    }
}

pub(crate) mod ui_context {
    use crate::prelude::*;
    use crate::window::miniquad::*;

    use crate::ui as megaui;

    use std::cell::RefCell;
    use std::rc::Rc;

    pub(crate) struct UiContext {
        pub ui: Rc<RefCell<megaui::Ui>>,
        ui_draw_list: Vec<megaui::DrawList>,
        material: Option<Material>,
    }

    impl UiContext {
        pub(crate) fn new(
            ctx: &mut dyn miniquad::RenderingBackend,
            screen_width: f32,
            screen_height: f32,
        ) -> UiContext {
            let ui = megaui::Ui::new(ctx, screen_width, screen_height);

            UiContext {
                ui: Rc::new(RefCell::new(ui)),
                ui_draw_list: vec![],
                material: None,
            }
        }

        pub(crate) fn process_input(&mut self) {
            use megaui::InputHandler;

            let mouse_position = mouse_position();

            let mut ui = self.ui.borrow_mut();
            ui.mouse_move(mouse_position);

            if is_mouse_button_pressed(MouseButton::Left) {
                ui.mouse_down(mouse_position);
            }
            if is_mouse_button_released(MouseButton::Left) {
                ui.mouse_up(mouse_position);
            }

            let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);

            while let Some(c) = get_char_pressed_ui() {
                if ctrl == false {
                    ui.char_event(c, false, false);
                }
            }

            macro_rules! process {
                ($code:tt) => {
                    if is_key_pressed(KeyCode::$code) || is_key_down(KeyCode::$code) {
                        ui.key_down(megaui::KeyCode::$code, shift, ctrl);
                    }
                };
            }

            process!(Up);
            process!(Down);
            process!(Right);
            process!(Left);
            process!(Home);
            process!(End);
            process!(Delete);
            process!(Backspace);
            process!(Tab);
            process!(Z);
            process!(Y);
            process!(C);
            process!(X);
            process!(V);
            process!(A);
            process!(Escape);
            process!(Enter);

            if is_key_down(KeyCode::LeftControl)
                || is_key_down(KeyCode::RightControl)
                || is_key_pressed(KeyCode::LeftControl)
                || is_key_pressed(KeyCode::RightControl)
            {
                ui.key_down(megaui::KeyCode::Control, shift, ctrl);
            }
            let (wheel_x, wheel_y) = mouse_wheel();
            ui.mouse_wheel(wheel_x, -wheel_y);
        }

        pub(crate) fn draw(
            &mut self,
            _ctx: &mut dyn miniquad::RenderingBackend,
            quad_gl: &mut QuadGl,
        ) {
            // TODO: this belongs to new and waits for cleaning up context initialization mess
            let material = self.material.get_or_insert_with(|| {
                load_material(
                    ShaderSource {
                        glsl_vertex: Some(VERTEX_SHADER),
                        glsl_fragment: Some(FRAGMENT_SHADER),
                        metal_shader: Some(METAL_SHADER),
                    },
                    MaterialParams {
                        pipeline_params: PipelineParams {
                            color_blend: Some(BlendState::new(
                                Equation::Add,
                                BlendFactor::Value(BlendValue::SourceAlpha),
                                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .unwrap()
            });

            let mut ui = self.ui.borrow_mut();
            self.ui_draw_list.clear();
            ui.render(&mut self.ui_draw_list);
            let mut ui_draw_list = vec![];

            std::mem::swap(&mut ui_draw_list, &mut self.ui_draw_list);

            let mut atlas = ui.atlas.lock().unwrap();
            let font_texture = atlas.texture();
            quad_gl.texture(Some(&Texture2D::unmanaged(font_texture)));

            gl_use_material(material);

            for draw_command in &ui_draw_list {
                if let Some(ref texture) = draw_command.texture {
                    quad_gl.texture(Some(texture));
                } else {
                    quad_gl.texture(Some(&Texture2D::unmanaged(font_texture)));
                }

                quad_gl.scissor(
                    draw_command
                        .clipping_zone
                        .map(|rect| (rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32)),
                );
                quad_gl.draw_mode(DrawMode::Triangles);
                quad_gl.geometry(&draw_command.vertices, &draw_command.indices);
            }
            quad_gl.texture(None);

            gl_use_default_material();

            std::mem::swap(&mut ui_draw_list, &mut self.ui_draw_list);

            drop(atlas);
            ui.new_frame(get_frame_time());
        }
    }

    pub struct ClipboardObject;

    impl megaui::ClipboardObject for ClipboardObject {
        fn get(&self) -> Option<String> {
            miniquad::window::clipboard_get()
        }

        fn set(&mut self, data: &str) {
            miniquad::window::clipboard_set(data)
        }
    }

    const VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec4 color0;
attribute vec2 texcoord;

varying lowp vec2 uv;
varying lowp vec2 pos;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
    color = color0 / 255.0;
}
";
    const FRAGMENT_SHADER: &'static str = "#version 100
varying lowp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}
";
    pub const METAL_SHADER: &str = r#"
#include <metal_stdlib>
    using namespace metal;

    struct Uniforms
    {
        float4x4 Model;
        float4x4 Projection;
    };

    struct Vertex
    {
        float3 position    [[attribute(0)]];
        float2 texcoord    [[attribute(1)]];
        float4 color0      [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = uniforms.Model * uniforms.Projection * float4(v.position, 1);
        out.color = v.color0 / 255.0;
        out.uv = v.texcoord;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
    }"#;
}
