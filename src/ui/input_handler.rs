#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyCode {
    Up,
    Down,
    Right,
    Left,
    Backspace,
    Delete,
    Enter,
    Tab,
    Home,
    End,
    Control,
    Escape,
    A, // select all
    Z, // undo
    Y, // redo
    C, // copy
    V, // paste
    X, // cut
}

pub trait InputHandler {
    fn mouse_down(&mut self, context: &mut crate::Context, position: (f32, f32));
    fn mouse_up(&mut self, context: &mut crate::Context, _: (f32, f32));
    fn mouse_wheel(&mut self, context: &mut crate::Context, x: f32, y: f32);
    fn mouse_move(&mut self, context: &mut crate::Context, position: (f32, f32));
    fn char_event(
        &mut self,
        context: &mut crate::Context,
        character: char,
        shift: bool,
        ctrl: bool,
    );
    fn key_down(
        &mut self,
        context: &mut crate::Context,
        key_down: KeyCode,
        shift: bool,
        ctrl: bool,
    );
}
