// TODO
#![allow(warnings)]

use crate::ui::{Id, Ui};

pub struct ComboBox<'a, 'b, 'c> {
    id: Id,
    label: &'a str,
    variants: &'b [&'c str],
}

impl<'a, 'b, 'c> ComboBox<'a, 'b, 'c> {
    pub fn new(id: Id, variants: &'b [&'c str]) -> ComboBox<'a, 'b, 'c> {
        ComboBox {
            id,
            label: "",
            variants,
        }
    }

    pub fn label<'x>(self, label: &'x str) -> ComboBox<'x, 'b, 'c> {
        ComboBox {
            id: self.id,
            variants: self.variants,
            label,
        }
    }

    pub fn ui(self, _ui: &mut Ui, _data: &mut usize) -> usize {
        // let mut context = ui.get_active_window_context();

        // let window_margin = context
        //     .style
        //     .window_style
        //     .background_margin
        //     .map_or(0.0, |x| x.left);

        // let size = Vec2::new(
        //     context.window.cursor.area.w
        //         - context.style.margin * 2.
        //         - context.window.cursor.ident
        //         - window_margin,
        //     19.,
        // );
        // let pos = context.window.cursor.fit(size, Layout::Vertical) + vec2(window_margin / 2., 0.0);

        // let active_area_w = size.x / 2.;
        // let triangle_area_w = 19.;

        // let text_measures = {
        //     let font = &mut *context.style.label_style.font.borrow_mut();
        //     let font_size = context.style.label_style.font_size;

        //     context
        //         .window
        //         .painter
        //         .label_size(&self.label, None, font, font_size)
        // };

        // let clickable_rect = Rect::new(pos.x, pos.y, active_area_w, size.y);

        // let (hovered, _) = context.register_click_intention(clickable_rect);

        // let state = context
        //     .storage_any
        //     .get_or_default::<bool>(hash!(self.id, "combobox_state"));

        // if context.window.was_active == false {
        //     *state = false;
        // }
        // context.window.painter.draw_rect(
        //     clickable_rect,
        //     context.style.editbox_background(context.focused),
        //     None,
        // );
        // {
        //     let font = &mut *context.style.label_style.font.borrow_mut();
        //     let font_size = context.style.label_style.font_size;

        //     context.window.painter.draw_label(
        //         self.variants[*data],
        //         Vec2::new(pos.x, pos.y + text_measures.offset_y),
        //         context.style.label_style.text_color,
        //         font,
        //         font_size,
        //     );
        // }

        // context.window.painter.draw_rect(
        //     Rect::new(
        //         pos.x + active_area_w - triangle_area_w,
        //         pos.y,
        //         triangle_area_w,
        //         size.y,
        //     ),
        //     context.style.editbox_background(context.focused),
        //     None,
        // );
        // context.window.painter.draw_triangle(
        //     Vec2::new(pos.x + active_area_w - triangle_area_w + 4.0, pos.y + 4.0),
        //     Vec2::new(pos.x + active_area_w - 4.0, pos.y + 4.0),
        //     Vec2::new(pos.x + active_area_w - triangle_area_w / 2.0, pos.y + 15.0),
        //     Color::new(0.7, 0.7, 0.7, 1.0),
        // );

        // {
        //     let font = &mut *context.style.label_style.font.borrow_mut();
        //     let font_size = context.style.label_style.font_size;

        //     context.window.painter.draw_label(
        //         self.label,
        //         Vec2::new(pos.x + size.x / 2. + 5., pos.y + text_measures.offset_y),
        //         context.style.label_style.text_color,
        //         font,
        //         font_size,
        //     );
        // }

        // let modal_size = Vec2::new(200.0, self.variants.len() as f32 * 20.0);
        // let modal_rect = Rect::new(pos.x, pos.y + 20.0, modal_size.x, modal_size.y);

        // if *state == false && context.focused && hovered && context.input.click_down {
        //     *state = true;
        // } else if *state
        //     && (context.input.escape
        //         || context.input.enter
        //         || (modal_rect.contains(context.input.mouse_position) == false
        //             && context.input.click_down))
        // {
        //     *state = false;
        // }

        // if *state {
        //     let context = ui.begin_modal(
        //         hash!("combobox", self.id),
        //         pos + Vec2::new(0., 20.),
        //         modal_size,
        //     );

        //     let state = context
        //         .storage_any
        //         .get_or_default::<bool>(hash!(self.id, "combobox_state"));

        //     for (i, variant) in self.variants.iter().enumerate() {
        //         let rect = Rect::new(
        //             pos.x + 5.0,
        //             pos.y + i as f32 * 20.0 + 20.0,
        //             active_area_w - 5.0,
        //             20.0,
        //         );
        //         let hovered = rect.contains(context.input.mouse_position);

        //         context.window.painter.draw_rect(
        //             rect,
        //             context.style.combobox_variant_border(hovered, *data == i),
        //             context
        //                 .style
        //                 .combobox_variant_background(hovered, *data == i),
        //         );

        //         let font = &mut *context.style.label_style.font.borrow_mut();
        //         let font_size = context.style.label_style.font_size;

        //         let text_measures = {
        //             context
        //                 .window
        //                 .painter
        //                 .label_size(variant, None, font, font_size)
        //         };

        //         context.window.painter.draw_label(
        //             variant,
        //             Vec2::new(
        //                 pos.x + 7.,
        //                 pos.y
        //                     + i as f32 * text_measures.height
        //                     + 20.0
        //                     + 2.0
        //                     + text_measures.offset_y,
        //             ),
        //             context.style.label_style.text_color,
        //             font,
        //             font_size,
        //         );

        //         if hovered && context.input.click_up {
        //             *data = i;
        //             *state = false;
        //         }
        //     }
        //     ui.end_modal();
        // }

        // *data
        unimplemented!()
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
