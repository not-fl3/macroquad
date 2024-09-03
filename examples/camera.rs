// Adventures of a 10 by 10 crab on the 100 by 100 field

use macroquad::window::next_frame;
use macroquad::{color::*, input::*, math::*, shapes::*};

fn draw_grid(canvas: &mut SpriteBatcher) {
    for i in 0..11 {
        canvas.draw(
            Line::new(vec2(0.0, -50.0), vec2(0.0, 50.0), 1.0),
            vec2(i as f32 * 10.0 - 50.0, 0.0),
            BLACK,
        );
        canvas.draw(
            Line::new(vec2(-50.0, 0.0), vec2(50.0, 0.0), 1.0),
            vec2(0.0, i as f32 * 10.0 - 50.0),
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
    let mut crab_position = vec2(0.0, 0.0);
    let mut canvas = ctx.new_canvas();
    // This will guarantee that no matter what window size,
    // the top to bottom distance will be 100 in world space.
    canvas.set_viewport_bound(ViewportBound::Horizontal(100.0));
    loop {
        ctx.clear_screen(WHITE);
        crab_position += crab_direction(&ctx);
        // crab_position will be exactly in the middle of the window
        canvas.set_viewport_center(crab_position);
        canvas.clear();
        draw_grid(&mut canvas);
        canvas.draw(
            Sprite::new(&crab).size(vec2(10.0, 10.0)),
            crab_position - vec2(5., 5.),
            WHITE,
        );
        ctx.blit_canvas(&mut canvas);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
