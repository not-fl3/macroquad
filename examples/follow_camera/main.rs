// Adventures of a 10 by 10 crab on the 200 by 200 field

use macroquad::window::next_frame;
use macroquad::{color::*, input::*, math::*, shapes::*};

mod camera;

fn draw_grid(canvas: &mut SpriteBatcher) {
    for i in 0..11 {
        canvas.draw(
            Line::new(vec2(0.0, -100.0), vec2(0.0, 100.0), 1.0),
            vec2(i as f32 * 20.0 - 100.0, 0.0),
            BLACK,
        );
        canvas.draw(
            Line::new(vec2(-100.0, 0.0), vec2(100.0, 0.0), 1.0),
            vec2(0.0, i as f32 * 20.0 - 100.0),
            BLACK,
        );
    }
}

fn crab_direction(ctx: &macroquad::Context) -> Vec2 {
    let mut dir = vec2(0.0, 0.0);
    if ctx.is_key_down(KeyCode::Up) {
        dir -= vec2(0.0, 0.8);
    }
    if ctx.is_key_down(KeyCode::Down) {
        dir += vec2(0.0, 0.8);
    }
    if ctx.is_key_down(KeyCode::Left) {
        dir -= vec2(0.8, 0.0);
    }
    if ctx.is_key_down(KeyCode::Right) {
        dir += vec2(0.8, 0.0);
    }
    dir
}

async fn game(ctx: macroquad::Context) {
    let crab = ctx
        .resources
        .load_texture("examples/ferris.png")
        .await
        .unwrap();
    let render_target = ctx.resources.render_target(250, 250).unwrap();
    let mut camera = camera::Camera::new(Rect::new(-100.0, -100.0, 200.0, 200.0));
    let mut crab_position = vec2(0.0, 0.0);
    let mut canvas = ctx.new_canvas();
    let mut ui_canvas = ctx.new_canvas();
    canvas.set_viewport_bound(ViewportBound::Horizontal(100.0));
    loop {
        if ctx.is_key_pressed(KeyCode::Z) {
            camera.shake_rotational(5.0, 10);
        }
        if ctx.is_key_pressed(KeyCode::X) {
            camera.shake_noise(1., 10, 1.0);
        }
        ctx.clear_screen(WHITE);
        ctx.clear_render_target(&render_target, WHITE);
        ui_canvas.clear();
        canvas.clear();
        crab_position += crab_direction(&ctx);
        let (p, r) = camera.update(canvas.viewport(None), crab_position);
        canvas.set_viewport_center(p);
        canvas.set_viewport_rotation(r);
        draw_grid(&mut canvas);
        canvas.draw(
            Sprite::new(&crab).size(vec2(10.0, 10.0)),
            crab_position - vec2(5., 5.),
            WHITE,
        );
        ctx.blit_canvas(&mut canvas);
        canvas.set_viewport_center(crab_position);
        canvas.set_viewport_rotation(0.0);
        ctx.blit_canvas2(&render_target, &mut canvas);
        ui_canvas.draw(Sprite::new(&render_target.texture), vec2(0.0, 0.0), WHITE);
        ctx.blit_canvas(&mut ui_canvas);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
