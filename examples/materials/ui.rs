use macroquad::{
    input::{KeyCode, MouseButton},
    math::{vec2, Rect},
    quad_gl::{
        color::{self, Color},
        texture::{Image, Texture2D},
        ui::{hash, Id, Ui},
    },
};

pub fn color_picker_texture(ctx: &macroquad::Context, w: usize, h: usize) -> (Texture2D, Image) {
    let ratio = 1.0 / h as f32;

    let mut image = Image::gen_image_color(w as u16, h as u16, color::WHITE);
    let image_data = image.get_image_data_mut();

    for j in 0..h {
        for i in 0..w {
            let lightness = 1.0 - i as f32 * ratio;
            let hue = j as f32 * ratio;

            image_data[i + j * w] = color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    (ctx.new_texture_from_image(&image), image)
}

pub fn color_picker(
    ctx: &macroquad::Context,
    ui: &mut Ui,
    id: Id,
    data: &mut Color,
    color_picker_texture: &Texture2D,
) -> bool {
    let is_mouse_captured = ui.is_mouse_captured();

    let mut canvas = ui.canvas();
    let cursor = canvas.request_space(vec2(200., 220.));
    let mouse = ctx.mouse_position();

    let x = mouse.x as i32 - cursor.x as i32;
    let y = mouse.y as i32 - (cursor.y as i32 + 20);

    if x > 0 && x < 200 && y > 0 && y < 200 {
        let ratio = 1.0 / 200.0 as f32;
        let lightness = 1.0 - x as f32 * ratio;
        let hue = y as f32 * ratio;

        if ctx.is_mouse_button_down(MouseButton::Left) && is_mouse_captured == false {
            *data = color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    canvas.rect(
        Rect::new(cursor.x - 5.0, cursor.y - 5.0, 210.0, 395.0),
        Color::new(0.7, 0.7, 0.7, 1.0),
        Color::new(0.9, 0.9, 0.9, 1.0),
    );

    canvas.rect(
        Rect::new(cursor.x, cursor.y, 200.0, 18.0),
        Color::new(0.0, 0.0, 0.0, 1.0),
        Color::new(data.r, data.g, data.b, 1.0),
    );
    canvas.image(
        Rect::new(cursor.x, cursor.y + 20.0, 200.0, 200.0),
        &color_picker_texture,
    );

    let (h, _, l) = color::rgb_to_hsl(*data);

    canvas.rect(
        Rect::new(
            cursor.x + (1.0 - l) * 200.0 - 3.5,
            cursor.y + h * 200. + 20.0 - 3.5,
            7.0,
            7.0,
        ),
        Color::new(0.3, 0.3, 0.3, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
    );

    ui.slider(hash!(id, "alpha"), "Alpha", 0.0..1.0, &mut data.a);
    ui.separator();
    ui.slider(hash!(id, "red"), "Red", 0.0..1.0, &mut data.r);
    ui.slider(hash!(id, "green"), "Green", 0.0..1.0, &mut data.g);
    ui.slider(hash!(id, "blue"), "Blue", 0.0..1.0, &mut data.b);
    ui.separator();
    let (mut h, mut s, mut l) = color::rgb_to_hsl(*data);
    ui.slider(hash!(id, "hue"), "Hue", 0.0..1.0, &mut h);
    ui.slider(hash!(id, "saturation"), "Saturation", 0.0..1.0, &mut s);
    ui.slider(hash!(id, "lightess"), "Lightness", 0.0..1.0, &mut l);
    let Color { r, g, b, .. } = color::hsl_to_rgb(h, s, l);
    data.r = r;
    data.g = g;
    data.b = b;

    ui.separator();
    if ui.button(None, "    ok    ")
        || ctx.is_key_down(KeyCode::Escape)
        || ctx.is_key_down(KeyCode::Enter)
        || (ctx.is_mouse_button_pressed(MouseButton::Left)
            && Rect::new(cursor.x - 10., cursor.y - 10.0, 230., 420.).contains(mouse) == false)
    {
        return true;
    }

    false
}

pub fn colorbox(
    ctx: &macroquad::Context,
    ui: &mut Ui,
    id: Id,
    label: &str,
    data: &mut Color,
    color_picker_texture: &Texture2D,
) {
    ui.label(None, label);
    let mut canvas = ui.canvas();
    let cursor = canvas.cursor();

    canvas.rect(
        Rect::new(cursor.x + 20.0, cursor.y, 50.0, 18.0),
        Color::new(0.2, 0.2, 0.2, 1.0),
        Color::new(data.r, data.g, data.b, 1.0),
    );
    if ui.last_item_clicked() {
        *ui.get_bool(hash!(id, "color picker opened")) ^= true;
    }
    if *ui.get_bool(hash!(id, "color picker opened")) {
        ui.popup(hash!(id, "color popup"), vec2(200., 400.), |ui| {
            if color_picker(&ctx, ui, id, data, color_picker_texture) {
                *ui.get_bool(hash!(id, "color picker opened")) = false;
            }
        });
    }
}
