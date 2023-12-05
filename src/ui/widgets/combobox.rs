use crate::{
    math::{vec2, Rect, Vec2},
    ui::{ElementState, Id, Layout, Ui, UiContent},
};

pub struct ComboBox<'a, 'b, 'c> {
    id: Id,
    label: &'a str,
    variants: &'b [&'c str],
    ratio: f32,
}

impl<'a, 'b, 'c> ComboBox<'a, 'b, 'c> {
    pub fn new(id: Id, variants: &'b [&'c str]) -> ComboBox<'a, 'b, 'c> {
        ComboBox {
            id,
            label: "",
            variants,
            ratio: 0.5,
        }
    }

    pub fn label<'x>(self, label: &'x str) -> ComboBox<'x, 'b, 'c> {
        ComboBox {
            id: self.id,
            variants: self.variants,
            label,
            ratio: self.ratio,
        }
    }

    pub fn ratio(self, ratio: f32) -> Self {
        Self { ratio, ..self }
    }
    pub fn ui(self, ui: &mut Ui, data: &mut usize) -> usize {
        let mut context = ui.get_active_window_context();

        let line_height = context.style.label_style.font_size;

        let size = vec2(
            context.window.cursor.area.w - context.style.margin * 2. - context.window.cursor.ident,
            (line_height as f32 + 4.).max(19.),
        );

        let combobox_area_w = size.x * self.ratio - 15.;

        let pos = context.window.cursor.fit(size, Layout::Vertical);

        let active_area_w = size.x * self.ratio;

        let text_measures = {
            let font = &mut *context.style.label_style.font.lock().unwrap();
            let font_size = context.style.label_style.font_size;

            context
                .window
                .painter
                .label_size(self.label, None, font, font_size)
        };

        let clickable_rect = Rect::new(pos.x, pos.y, active_area_w, size.y);

        let (hovered, _) = context.register_click_intention(clickable_rect);

        let state = context
            .storage_any
            .get_or_default::<bool>(hash!(self.id, "combobox_state"));

        if context.window.was_active == false {
            *state = false;
        }

        context.window.painter.draw_element_background(
            &context.style.combobox_style,
            pos,
            vec2(combobox_area_w, size.y),
            ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                ..Default::default()
            },
        );

        context.window.painter.draw_element_content(
            &context.style.label_style,
            pos,
            vec2(combobox_area_w, size.y),
            &UiContent::Label((&*self.variants[*data]).into()),
            ElementState {
                focused: context.focused,
                hovered,
                clicked: hovered && context.input.is_mouse_down,
                selected: false,
            },
        );

        {
            context.window.painter.draw_element_label(
                &context.style.label_style,
                Vec2::new(pos.x + size.x * self.ratio, pos.y),
                self.label,
                ElementState {
                    focused: context.focused,
                    ..Default::default()
                },
            );
        }

        let modal_size = Vec2::new(active_area_w, self.variants.len() as f32 * size.y);
        let modal_rect = Rect::new(pos.x, pos.y + size.y, modal_size.x, modal_size.y);

        if *state == false && context.focused && hovered && context.input.click_down {
            *state = true;
        } else if *state
            && (context.input.escape
                || context.input.enter
                || (modal_rect.contains(context.input.mouse_position) == false
                    && context.input.click_down))
        {
            *state = false;
        }

        if *state {
            let context = ui.begin_modal(
                hash!("combobox", self.id),
                pos + Vec2::new(0., 20.),
                modal_size,
            );

            let state = context
                .storage_any
                .get_or_default::<bool>(hash!(self.id, "combobox_state"));

            for (i, variant) in self.variants.iter().enumerate() {
                let rect = Rect::new(
                    pos.x + 5.0,
                    pos.y + i as f32 * size.y + size.y,
                    active_area_w - 5.0,
                    size.y,
                );
                let hovered = rect.contains(context.input.mouse_position);

                let color = context.style.combobox_style.color(ElementState {
                    focused: context.focused,
                    hovered,
                    clicked: hovered && context.input.is_mouse_down,
                    selected: false,
                });

                context.window.painter.draw_rect(
                    rect, //context.style.combobox_variant_border(hovered, *data == i),
                    color,
                    // context
                    //     .style
                    //     .combobox_variant_background(hovered, *data == i),
                    color,
                );

                let font = &mut *context.style.label_style.font.lock().unwrap();
                let font_size = context.style.label_style.font_size;

                context.window.painter.draw_label(
                    variant,
                    Vec2::new(
                        pos.x + 7.,
                        pos.y + i as f32 * size.y + size.y + 2.0 + text_measures.offset_y,
                    ),
                    context.style.combobox_style.text_color,
                    font,
                    font_size,
                );

                if hovered && context.input.click_up {
                    *data = i;
                    *state = false;
                }
            }
            ui.end_modal();
        }

        *data
    }
}

impl Ui {
    pub fn combo_box<'a>(
        &mut self,
        id: Id,
        label: &str,
        variants: &[&str],
        data: impl Into<Option<&'a mut usize>>,
    ) -> usize {
        if let Some(r) = data.into() {
            ComboBox::new(id, variants).label(label).ui(self, r)
        } else {
            let data_id = hash!(id, "selected_variant");
            let mut selected_variant = { *self.get_any(data_id) };

            ComboBox::new(id, variants)
                .label(label)
                .ui(self, &mut selected_variant);

            *self.get_any(data_id) = selected_variant;

            selected_variant
        }
    }
}
