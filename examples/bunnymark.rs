use macroquad::{
    camera::{Camera, Environment, Projection},
    color::*,
    math::{vec2, vec3},
    shapes::{DrawParams, Sprite, Text},
};
use miniquad::KeyCode;

use macroquad::compat::*;

async fn game(ctx: macroquad::Context) {
    let mut scene = ctx.new_scene();

    let texture = ctx
        .resources
        .load_texture("examples/ferris2.png")
        .await
        .unwrap();
    let sprite = Sprite::new(&texture);
    let bunny = scene.into_model(sprite, DrawParams::default());
    let bunny = scene.add_model(&bunny);
    scene.set_scale(&bunny, vec3(0.01, 1.0, 0.01));
    let mut camera = Camera {
        environment: Environment::SolidColor(WHITE),
        depth_enabled: false,
        projection: Projection::Orthographic,
        position: vec3(0., 10.0, 0.),
        up: vec3(0., 1., 0.),
        target: vec3(0.0, 0., 1.),
        z_near: 0.1,
        z_far: 15.0,
        ..Default::default()
    };

    let mut canvas = ctx.new_canvas();
    let mut bunnies = vec![];
    let mut bunnies_dir = vec![];

    for _ in 0..1 {
        bunnies.push(vec3(
            quad_rand::gen_range(-20.0, 20.0),
            0.0,
            quad_rand::gen_range(-20.0, 20.0),
        ));
        bunnies_dir.push(vec3(
            quad_rand::gen_range(-1.0, 1.0),
            0.,
            quad_rand::gen_range(-1.0, 1.0),
        ));
    }
    scene.update_multi_positions(&bunny, &bunnies);

    loop {
        if ctx.is_key_down(KeyCode::Space) {
            for _ in 0..1 {
                bunnies.push(vec3(
                    quad_rand::gen_range(-20.0, 20.0),
                    0.0,
                    quad_rand::gen_range(-20.0, 20.0),
                ));
                bunnies_dir.push(vec3(
                    quad_rand::gen_range(-1.0, 1.0),
                    0.,
                    quad_rand::gen_range(-1.0, 1.0),
                ));
            }
            scene.update_multi_positions(&bunny, &bunnies);
        }

        ctx.clear_screen(WHITE);

        for (bunny, dir) in bunnies.iter_mut().zip(bunnies_dir.iter_mut()) {
            *bunny += *dir;
            if bunny.x >= 20.0 || bunny.x <= -20.0 {
                dir.x *= -1.0;
            }
            if bunny.z >= 20.0 || bunny.z <= -20.0 {
                dir.z *= -1.0;
            }
        }

        scene.update_multi_positions(&bunny, &bunnies);

        scene.draw(&mut camera);

        canvas.clear();
        canvas.draw(
            Text::new(&format!("fps: {:0.1}", 1.0 / ctx.frame_time()), 16),
            vec2(0.0, 16.0),
            BLACK,
        );
        canvas.draw(
            Text::new(&format!("bunnies: {}", bunnies.len()), 16),
            vec2(0.0, 32.0),
            BLACK,
        );
        canvas.draw(
            Text::new(&format!("Press any key"), 16),
            vec2(0.0, 48.0),
            BLACK,
        );
        canvas.blit();

        next_frame().await;
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
