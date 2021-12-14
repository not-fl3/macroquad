use crate::{
    math::{vec2, Rect},
    ui::{ElementState, Id, Layout, Ui},
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

    pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) -> bool {
        if let Some(token) = self.begin(ui) {
            f(ui);
            token.end(ui)
        } else {
            false
        }
    }

    pub fn begin(self, ui: &mut Ui) -> Option<TreeNodeToken> {
        let context = ui.get_active_window_context();

        let size = vec2(300., 14.);

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

        context.window.painter.draw_element_label(
            &context.style.label_style,
            pos,
            if *opened == 0 { "+" } else { "-" },
            ElementState {
                focused: context.focused,
                ..Default::default()
            },
        );
        context.window.painter.draw_element_label(
            &context.style.label_style,
            pos + vec2(10., 0.),
            &*self.label,
            ElementState {
                focused: context.focused,
                ..Default::default()
            },
        );

        if *opened == 1 {
            context.window.cursor.ident += 5.;

            Some(TreeNodeToken { clicked })
        } else {
            None
        }
    }
}

#[must_use = "Must call `.end()` to finish TreeNode"]
pub struct TreeNodeToken {
    clicked: bool,
}

impl TreeNodeToken {
    pub fn end(self, ui: &mut Ui) -> bool {
        let context = ui.get_active_window_context();
        context.window.cursor.ident -= 5.;

        self.clicked
    }
}

impl Ui {
    pub fn tree_node<F: FnOnce(&mut Ui)>(&mut self, id: Id, label: &str, f: F) -> bool {
        TreeNode::new(id, label).ui(self, f)
    }
}
