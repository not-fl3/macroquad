use std::collections::HashMap;

use macroquad::prelude::*;

struct UiContext {
    ui: megaui::Ui,
    ui_draw_list: Vec<megaui::DrawList>,
    font_texture: Texture2D,
    megaui_textures: HashMap<u32, Texture2D>,
    input_processed_this_frame: bool,
}

static mut UI_CONTEXT: Option<UiContext> = None;

impl UiContext {
    fn new(ctx: &mut miniquad::Context) -> UiContext {
        let ui = megaui::Ui::new();

        let texture_data = &ui.font_atlas.texture;
        let font_texture = Texture2D::from_rgba8(
            ctx,
            texture_data.width as u16,
            texture_data.height as u16,
            &texture_data.data,
        );
        font_texture.set_filter(ctx, FilterMode::Nearest);

        UiContext {
            ui,
            ui_draw_list: vec![],
            font_texture,
            megaui_textures: HashMap::new(),
            input_processed_this_frame: false,
        }
    }

    fn get() -> &'static mut UiContext {
        unsafe {
            if UI_CONTEXT.is_none() {
                let InternalGlContext {
                    quad_context: ctx, ..
                } = get_internal_gl();

                UI_CONTEXT = Some(UiContext::new(ctx));
            }

            UI_CONTEXT.as_mut().unwrap()
        }
    }
}

pub struct ClipboardObject;

impl megaui::ClipboardObject for ClipboardObject {
    fn get(&self) -> Option<String> {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        miniquad::clipboard::get(ctx)
    }

    fn set(&mut self, data: &str) {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        miniquad::clipboard::set(ctx, data)
    }
}

pub struct WindowParams {
    pub label: String,
    pub movable: bool,
    pub close_button: bool,
    pub titlebar: bool,
}

impl Default for WindowParams {
    fn default() -> WindowParams {
        WindowParams {
            label: "".to_string(),
            movable: true,
            close_button: false,
            titlebar: true,
        }
    }
}

pub fn set_ui_style(style: megaui::Style) {
    let ctx = UiContext::get();

    ctx.ui.set_style(style);
}

pub fn set_megaui_texture(id: u32, texture: Texture2D) {
    let ctx = UiContext::get();

    ctx.megaui_textures.insert(id, texture);
}

pub fn draw_window<F: FnOnce(&mut megaui::Ui)>(
    id: megaui::Id,
    position: glam::Vec2,
    size: glam::Vec2,
    params: impl Into<Option<WindowParams>>,
    f: F,
) -> bool {
    let ctx = UiContext::get();

    process_input();

    let ui = &mut ctx.ui;
    let params = params.into();

    megaui::widgets::Window::new(
        id,
        megaui::Vector2::new(position.x(), position.y()),
        megaui::Vector2::new(size.x(), size.y()),
    )
    .label(params.as_ref().map_or("", |params| &params.label))
    .titlebar(params.as_ref().map_or(true, |params| params.titlebar))
    .movable(params.as_ref().map_or(true, |params| params.movable))
    .close_button(params.as_ref().map_or(false, |params| params.close_button))
    .ui(ui, f)
}

/// Check for megaui mouse overlap
pub fn mouse_over_ui() -> bool {
    let mouse_position = mouse_position();

    UiContext::get()
        .ui
        .is_mouse_over(megaui::Vector2::new(mouse_position.0, mouse_position.1))
}

fn process_input() {
    use megaui::InputHandler;

    let mut ctx = UiContext::get();

    if ctx.input_processed_this_frame {
        return;
    }
    let mouse_position = mouse_position();

    ctx.ui.mouse_move(mouse_position);

    if is_mouse_button_pressed(MouseButton::Left) {
        ctx.ui.mouse_down(mouse_position);
    }
    if is_mouse_button_released(MouseButton::Left) {
        ctx.ui.mouse_up(mouse_position);
    }

    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);

    while let Some(c) = get_key_pressed() {
        if ctrl == false {
            ctx.ui.char_event(c, false, false);
        }
    }

    if is_key_pressed(KeyCode::Up) {
        ctx.ui.key_down(megaui::KeyCode::Up, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Down) {
        ctx.ui.key_down(megaui::KeyCode::Down, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Right) {
        ctx.ui.key_down(megaui::KeyCode::Right, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Left) {
        ctx.ui.key_down(megaui::KeyCode::Left, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Home) {
        ctx.ui.key_down(megaui::KeyCode::Home, shift, ctrl);
    }
    if is_key_pressed(KeyCode::End) {
        ctx.ui.key_down(megaui::KeyCode::End, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Delete) {
        ctx.ui.key_down(megaui::KeyCode::Delete, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Backspace) {
        ctx.ui.key_down(megaui::KeyCode::Backspace, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Enter) {
        ctx.ui.key_down(megaui::KeyCode::Enter, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Tab) {
        ctx.ui.key_down(megaui::KeyCode::Tab, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Z) {
        ctx.ui.key_down(megaui::KeyCode::Z, shift, ctrl);
    }
    if is_key_pressed(KeyCode::Y) {
        ctx.ui.key_down(megaui::KeyCode::Y, shift, ctrl);
    }
    if is_key_pressed(KeyCode::C) {
        ctx.ui.key_down(megaui::KeyCode::C, shift, ctrl);
    }
    if is_key_pressed(KeyCode::X) {
        ctx.ui.key_down(megaui::KeyCode::X, shift, ctrl);
    }
    if is_key_pressed(KeyCode::V) {
        ctx.ui.key_down(megaui::KeyCode::V, shift, ctrl);
    }
    if is_key_pressed(KeyCode::A) {
        ctx.ui.key_down(megaui::KeyCode::A, shift, ctrl);
    }

    ctx.input_processed_this_frame = true;
}

/// Tick megaui state and draw everything
/// Should be called once per frame at the end of the frame
pub fn draw() {
    let mut ctx = UiContext::get();

    ctx.input_processed_this_frame = false;

    let InternalGlContext { quad_gl, .. } = unsafe { get_internal_gl() };

    ctx.ui_draw_list.clear();

    ctx.ui.render(&mut ctx.ui_draw_list);
    let mut ui_draw_list = vec![];

    std::mem::swap(&mut ui_draw_list, &mut ctx.ui_draw_list);

    quad_gl.texture(Some(ctx.font_texture));

    for draw_command in &ui_draw_list {
        if let Some(texture) = draw_command.texture {
            quad_gl.texture(Some(ctx.megaui_textures[&texture]));
        } else {
            quad_gl.texture(Some(ctx.font_texture));
        }
        quad_gl.scissor(
            draw_command
                .clipping_zone
                .map(|rect| (rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32)),
        );
        quad_gl.draw_mode(DrawMode::Triangles);
        quad_gl.geometry(&draw_command.vertices, &draw_command.indices);
    }
    quad_gl.texture(None);

    std::mem::swap(&mut ui_draw_list, &mut ctx.ui_draw_list);

    ctx.ui.new_frame();
}
