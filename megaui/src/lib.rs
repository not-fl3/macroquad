mod draw_command;
mod draw_list;
mod input_handler;
mod style;
mod types;
mod ui;
mod canvas;

pub mod widgets;

pub use draw_list::{DrawList, Vertex};
pub use input_handler::{InputHandler, KeyCode};
pub use style::Style;
pub use types::{Color, Rect, Vector2};
pub use ui::{Drag, Layout, Ui};

pub type Id = u64;

#[macro_export]
macro_rules! hash {
    ($s:expr) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id = $s;

        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        s.finish()
    }};
    () => {{
        let id = concat!(file!(), line!(), column!());
        hash!(id)
    }};
    ($($s:expr),*) => {{
        let mut s: u128 = 0;
        $(s += $crate::hash!($s) as u128;)*
        $crate::hash!(s)
    }};
}

pub trait ClipboardObject {
    fn get(&self) -> Option<String>;
    fn set(&mut self, data: &str);
}

pub(crate) struct LocalClipboard {
    data: String
}

impl LocalClipboard {
    fn new() -> LocalClipboard {
        LocalClipboard {
            data: String::new()
        }
    }
}
impl ClipboardObject for LocalClipboard {
    fn get(&self) -> Option<String> {
        Some(self.data.clone())
    }

    fn set(&mut self, data: &str) {
        self.data = data.to_owned();
    }
}
