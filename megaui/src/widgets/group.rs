use crate::{
    types::{Rect, Vector2},
    ui::{Drag, DragState},
    Id, Layout, Ui,
};

#[derive(Debug, Clone)]
pub struct Group {
    id: Id,
    position: Option<Vector2>,
    layout: Layout,
    size: Vector2,
    draggable: bool,
    highlight: bool,
    hoverable: bool,
}

impl Group {
    pub fn new(id: Id, size: Vector2) -> Group {
        Group {
            id,
            size,
            position: None,
            layout: Layout::Horizontal,
            draggable: false,
            highlight: false,
            hoverable: false,
        }
    }

    pub fn position(self, position: Vector2) -> Group {
        Group {
            position: Some(position),
            ..self
        }
    }

    pub fn layout(self, layout: Layout) -> Group {
        Group { layout, ..self }
    }

    pub fn draggable(self, draggable: bool) -> Group {
        Group { draggable, ..self }
    }

    pub fn hoverable(self, hoverable: bool) -> Group {
        Group { hoverable, ..self }
    }

    pub fn highlight(self, highlight: bool) -> Group {
        Group { highlight, ..self }
    }

    pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) -> Drag {
        let mut drag = Drag::No;

        let parent = ui.get_active_window_context();

        let parent_rect = parent.window.content_rect();

        parent.window.childs.push(self.id);

        let pos = parent.window.cursor.fit(
            self.size,
            self.position.map_or(self.layout, Layout::Free),
        );
        let rect = Rect::new(pos.x, pos.y, self.size.x, self.size.y);
        let parent_id = Some(parent.window.id);

        let mut context = ui.begin_window(self.id, parent_id, pos, self.size, 0., true);

        let hovered =
            (self.hoverable || self.draggable) && rect.contains(context.input.mouse_position);

        if self.draggable && context.dragging.is_none() && hovered && context.input.click_down {
            *context.dragging = Some((self.id, DragState::Clicked(context.input.mouse_position)));
        }

        if let Some((id, DragState::Clicked(orig))) = context.dragging {
            if *id == self.id
                && context.input.is_mouse_down
                && context.input.mouse_position.distance(*orig) > 5.
            {
                *context.dragging = Some((self.id, DragState::Dragging(*orig)));
            }
            if context.input.is_mouse_down == false {
                *context.dragging = None;
            }
        }

        if let Some((id, DragState::Dragging(_))) = context.dragging {
            let id = *id;

            if id == self.id {
                drag = Drag::Dragging(context.input.mouse_position, *context.drag_hovered_previous_frame);

                if context.input.is_mouse_down == false {
                    *context.dragging = None;
                    drag = Drag::Dropped(context.input.mouse_position, *context.drag_hovered_previous_frame);
                }
            }

            if id != self.id && hovered {
                *context.drag_hovered = Some(self.id);
            }
        }

	context.window.draw_commands.clip(parent_rect);

	context.scroll_area();

        let clip_rect = context.window.content_rect();
        context.window.draw_commands.clip(clip_rect);
        context.window.draw_commands.draw_rect(
            rect,
            context
                .global_style
                .drag_border(context.focused, hovered, self.highlight),
            None,
        );

        f(ui);

        let context = ui.get_active_window_context();

        context.window.draw_commands.clip(None);

        if context.focused && self.draggable {
            if
            //parent.dragging.is_none()
            context.input.is_mouse_down
                && Rect::new(pos.x, pos.y, self.size.x, self.size.y)
                    .contains(context.input.mouse_position)
            {
                // *context.dragging = Some((
                //     id,
                //     DragState::Clicked(context.input.mouse_position, Vector2::new(rect.x, rect.y)),
                // ));
            }
        }

        ui.end_window();

        drag
    }
}

impl Ui {
    pub fn group<F: FnOnce(&mut Ui)>(&mut self, id: Id, size: Vector2, f: F) -> Drag {
        Group::new(id, size).ui(self, f)
    }
}
