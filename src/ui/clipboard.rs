pub trait ClipboardObject {
    fn get(&self) -> Option<String>;
    fn set(&mut self, data: &str);
}

pub(crate) struct LocalClipboard {
    data: String,
}

impl LocalClipboard {
    pub fn new() -> LocalClipboard {
        LocalClipboard {
            data: String::new(),
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
