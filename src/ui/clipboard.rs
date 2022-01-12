pub trait ClipboardObject {
    fn get(&self, context: &mut crate::Context) -> Option<String>;
    fn set(&mut self, context: &mut crate::Context, data: &str);
}
