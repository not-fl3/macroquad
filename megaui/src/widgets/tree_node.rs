use crate::{
    types::{Rect, Vector2},
    Id, Layout, Ui,
};

use std::borrow::Cow;

pub struct TreeNode<'a> {
    id: Id,
    label: Cow<'a, str>,
    init_unfolded: bool,
}

impl<'a> TreeNode<'a> {
    pub fn new<S>(id: Id, label: S) -> TreeNode<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        TreeNode {
            id,
            label: label.into(),
            init_unfolded: false,
        }
    }

    pub fn init_unfolded(mut self) -> TreeNode<'a> {
	self.init_unfolded = true;
	self
    }

    pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) {
        let context = ui.get_active_window_context();

        let size = Vector2::new(300., 14.);

        let color = context.global_style.text(context.focused);
        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
        let hovered = rect.contains(context.input.mouse_position);

        let clicked = context.focused && hovered && context.input.click_down();

        let opened = context
            .storage_u32
            .entry(self.id)
            .or_insert(if self.init_unfolded { 1 } else { 0 });

        if clicked {
            *opened ^= 1;
        }

        context
            .window
            .draw_commands
            .draw_label(if *opened == 0 { "+" } else { "-" }, pos, color);
        context
            .window
            .draw_commands
            .draw_label(&*self.label, pos + Vector2::new(10., 0.), color);

        if *opened == 1 {
            context.window.cursor.ident += 5.;

            f(ui);

            let context = ui.get_active_window_context();
            context.window.cursor.ident -= 5.;
        }
    }
}

impl Ui {
    pub fn tree_node<F: FnOnce(&mut Ui)>(&mut self, id: Id, label: &str, f: F) {
        TreeNode::new(id, label).ui(self, f)
    }
}
