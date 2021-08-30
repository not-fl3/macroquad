use crate::{
    math::{vec2, Rect, Vec2},
    ui::{widgets::Editbox, ElementState, Id, Layout, Ui, UiContent},
};

use std::any::Any;

pub trait Num:
    Copy + std::string::ToString + std::str::FromStr + Into<f64> + 'static + Default + std::fmt::Display
{
}

#[derive(Clone, Copy)]
struct DragState {
    start_value: f64,
    start_mouse: f32,
}

struct State {
    string_represents: f64,
    string: String,
    before: String,
    drag: Option<DragState>,
    in_editbox: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            string_represents: 0.0,
            string: String::new(),
            before: String::new(),
            drag: None,
            in_editbox: false,
        }
    }
}

pub struct Drag<'a> {
    id: Id,
    label: &'a str,
    range: Option<(f64, f64)>,
    size: Option<Vec2>,
    step: f32,
}

impl<'a> Drag<'a> {
    pub fn new(id: Id) -> Drag<'a> {
        Drag {
            id,
            size: None,
            range: None,
            label: "",
            step: 0.1,
        }
    }

    pub fn label<'b>(self, label: &'b str) -> Drag<'b> {
        Drag {
            label,
            id: self.id,
            range: self.range,
            size: self.size,
            step: self.step,
        }
    }

    pub fn range<T: Num>(self, range: Option<(T, T)>) -> Drag<'a> {
        Drag {
            range: range.map(|(start, end)| (start.into(), end.into())),
            ..self
        }
    }
    // /// Ratio of pixels on the x-axis dragged to how much the value should be changed.
    // /// Default for floating point numbers is `0.1`, for integers it's `1`.
    // pub fn step(self, step: f32) -> Self {
    //     Self { step, ..self }
    // }

    pub fn ui<T>(self, ui: &mut Ui, data: &mut T)
    where
        T: std::any::Any + Num,
    {
        let context = ui.get_active_window_context();
        let state_hash = hash!(self.id, "input_float_state");
        let mut s: State = std::mem::take(context.storage_any.get_or_default(state_hash));

        let label_size = context.window.painter.content_with_margins_size(
            &context.style.label_style,
            &UiContent::Label(self.label.into()),
        );
        let size = vec2(
            context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
            label_size.y.max(22.),
        );

        let pos = context.window.cursor.fit(size, Layout::Vertical);
        let editbox_area = Vec2::new(
            if self.label.is_empty() {
                size.x
            } else {
                size.x / 2.0
            },
            size.y,
        );
        let hovered = Rect::new(pos.x, pos.y, editbox_area.x, editbox_area.y)
            .contains(context.input.mouse_position);

        // state transition between editbox and dragbox
        if s.in_editbox == false {
            if hovered && context.input.is_mouse_down() && context.input.modifier_ctrl {
                s.in_editbox = true;
            }
        } else {
            if context.input.escape
                || context.input.enter
                || (hovered == false && context.input.is_mouse_down())
            {
                s.in_editbox = false;
            }
        }

        if s.in_editbox == false {
            let context = ui.get_active_window_context();

            // context.window.painter.draw_rect(
            //     Rect::new(pos.x, pos.y, editbox_area.x, editbox_area.y),
            //     None,
            //     context.style.drag_background(context.focused),
            // );

            let label = format!("{:.2}", (*data));
            let value_size = context.window.painter.content_with_margins_size(
                &context.style.label_style,
                &UiContent::Label((&label).into()),
            );

            context.window.painter.draw_element_label(
                &context.style.label_style,
                pos + Vec2::new(size.x / 2. - value_size.x - 15., 0.),
                &label,
                ElementState {
                    focused: context.focused,
                    hovered: false,
                    clicked: false,
                    selected: false,
                },
            );

            if let Some(drag) = s.drag {
                if context.input.is_mouse_down == false {
                    s.drag = None;
                    context.input.cursor_grabbed = false;
                    if !hovered {
                        *context.input_focus = None;
                    }
                } else {
                    let mouse_delta =
                        (context.input.mouse_position.x - drag.start_mouse) * self.step;

                    if (data as &mut dyn Any).is::<f32>() {
                        let data = (data as &mut dyn Any).downcast_mut::<f32>().unwrap();
                        *data = drag.start_value as f32 + mouse_delta;
                        if let Some((start, end)) = self.range {
                            *data = data.max(start as f32).min(end as f32);
                        }
                    }
                    if (data as &mut dyn Any).is::<u32>() {
                        let data = (data as &mut dyn Any).downcast_mut::<u32>().unwrap();
                        *data = (drag.start_value as i32 + mouse_delta as i32).max(0) as u32;
                        if let Some((start, end)) = self.range {
                            *data = (*data).max(start as u32).min(end as u32);
                        }
                    }
                }
            } else {
                if hovered && context.input.is_mouse_down() {
                    s.drag = Some(DragState {
                        start_mouse: context.input.mouse_position.x,
                        start_value: (*data).into(),
                    });
                    *context.input_focus = Some(self.id);
                    context.input.cursor_grabbed = true;
                }
            }
        } else {
            if s.string_represents != (*data).into() {
                s.string = data.to_string();
            }

            Editbox::new(self.id, editbox_area)
                .position(pos)
                .multiline(false)
                .ui(ui, &mut s.string);

            if let Ok(n) = s.string.parse() {
                *data = n;
                s.string_represents = n.into();
                s.before = s.string.clone();
            } else if s.string.is_empty() {
                *data = T::default();
                s.string_represents = 0.0;
                s.before = s.string.clone();
            } else {
                s.string = s.before.clone();
            }
        }

        let context = ui.get_active_window_context();

        if self.label.is_empty() == false {
            context.window.painter.draw_element_label(
                &context.style.label_style,
                Vec2::new(pos.x + size.x / 2. + 5., pos.y),
                self.label,
                ElementState {
                    focused: context.focused,
                    hovered: false,
                    clicked: false,
                    selected: false,
                },
            );
        }

        *context.storage_any.get_or_default(state_hash) = s;
    }
}

impl Num for u32 {}
impl Num for f32 {}

impl Ui {
    pub fn drag<T: Num, T1: Into<Option<(T, T)>>>(
        &mut self,
        id: Id,
        label: &str,
        range: T1,
        data: &mut T,
    ) {
        let range = range.into();

        Drag::new(id).label(label).range(range).ui(self, data)
    }
}
