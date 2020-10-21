use crate::{
    types::{Rect, Color, Vector2},
	hash,
    widgets::Editbox,
    Id, Layout, Ui,
};

#[derive(Default)]
struct State {
	string_represents: f32,
	string: String,
	before: String,
	drag: Option<Drag>,
}

#[derive(Clone, Copy)]
struct Drag {
	start_mouse: Vector2,
	start_data: f32,
}

pub struct InputFloat<'a> {
    id: Id,
    label: &'a str,
    size: Option<Vector2>,
	step: f32,
}

impl<'a> InputFloat<'a> {
    pub fn new(id: Id) -> InputFloat<'a> {
        InputFloat {
            id,
            size: None,
            label: "",
			step: 0.1,
        }
    }

    pub fn label<'b>(self, label: &'b str) -> InputFloat<'b> {
        InputFloat {
            label,
            id: self.id,
            size: self.size,
            step: self.step,
        }
    }

    pub fn size(self, size: Vector2) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

	/// Ratio of pixels on the x-axis dragged to how much the value should be changed.
	/// Default is `0.1`.
    pub fn step(self, step: f32) -> Self {
        Self {
            step,
            ..self
        }
    }

    pub fn ui(self, ui: &mut Ui, data: &mut f32) {
        let context = ui.get_active_window_context();
		let state_hash = hash!(self.id, "input_float_state");
        let mut s: State = std::mem::take(
			context
				.storage_any
				.get_or_default(state_hash)
		);

        let size = self.size.unwrap_or_else(|| {
            Vector2::new(
                context.window.cursor.area.w
                    - context.global_style.margin * 2.
                    - context.window.cursor.ident,
                19.,
            )
        });
        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let editbox_area = Vector2::new(
			if self.label.is_empty() { size.x } else { size.x / 2.0 },
			size.y,
		);

        let editbox = Editbox::new(self.id, editbox_area)
            .position(pos)
            .multiline(false);

        let hovered = Rect::new(pos.x, pos.y, editbox_area.x, editbox_area.y)
            .contains(context.input.mouse_position);

        if hovered && context.input.is_mouse_down() {
            s.drag = Some(Drag {
                start_mouse: context.input.mouse_position,
                start_data: *data,
            });
            context.window.input_focus = Some(self.id);
            context.input.cursor_grabbed = true;
        }

        if let Some(drag) = s.drag {
            if context.input.is_mouse_down == false {
                s.drag = None;
                context.input.cursor_grabbed = false;
                if !hovered {
                    context.window.input_focus = None;
                } else {
                    use super::editbox::EditboxState;
                    context
                        .storage_any
                        .get_or_default::<EditboxState>(hash!(self.id, "cursor"))
                        .select_all(&mut s.string)
                }
            }

            let mouse_delta = context.input.mouse_position.x - drag.start_mouse.x;
            *data = drag.start_data + mouse_delta * self.step;
        }

		if s.string_represents != *data {
			s.string = data.to_string();
		}

        editbox.ui(ui, &mut s.string);

        if let Ok(n) = s.string
            .parse()
            .or_else(|e| if s.string.is_empty() { Ok(0.0) } else { Err(e) })
        {
            *data = n;
            s.string_represents = n;
            s.before = s.string.clone();
        } else {
            s.string = s.before.clone();
        }

        let context = ui.get_active_window_context();

        if self.label.is_empty() == false {
            context.window.draw_commands.draw_label(
                self.label,
                Vector2::new(pos.x + size.x / 2. + 5., pos.y + 2.),
                Color::from_rgba(0, 0, 0, 255),
            );
        }

        *context.storage_any.get_or_default(state_hash) = s;
    }
}

impl Ui {
    pub fn input_float(&mut self, id: Id, label: &str, data: &mut f32) {
        InputFloat::new(id).label(label).ui(self, data)
    }
}
