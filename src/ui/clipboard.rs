pub trait ClipboardObject {
    fn get(&self) -> Option<String>;
    fn set(&mut self, data: &str);
}
