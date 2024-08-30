use macroquad::{
    camera::{Camera, Environment, Projection},
    color::*,
    math::{vec2, vec3, Quat},
    scene::{ShadowCaster, ShadowSplit},
    shapes::{Circle, DrawParams, Sprite, Text},
};
use miniquad::KeyCode;

use macroquad::compat::*;

async fn game(ctx: macroquad::Context) {
    let mut scene = ctx.new_scene();

    scene.add_shadow_caster(ShadowCaster {
        direction: vec3(1.5, 1.5, 1.0),
        split: ShadowSplit::PSSM4,
    });
    let plane = ctx.mesh(macroquad::models::square(), None);
    let plane = scene.add_model(&plane);
    {
        scene.set_translation(&plane, vec3(0., 0.0, 0.));
        scene.set_scale(&plane, vec3(100.0, 1.0, 100.));
    }

    let texture = ctx
        .resources
        .load_texture("examples/ferris2.png")
        .await
        .unwrap();
    let text = scene.model(
        Text {
            font_scale: 0.1,
            ..Text::new("HELLO WORLD", 300)
        },
        DrawParams::default(),
    );
    let text = scene.add_model(&text);
    scene.set_translation(&text, vec3(-40.0, 0.0, 0.0));
    scene.set_scale(&text, vec3(0.5, 1.0, 0.5));

    // let text = scene.model(Circle::new(1.0), RED);
    // let text = scene.add_model(&text);
    // scene.set_scale(&text, vec3(1.0, 1.0, 1.0));
    // let bunny = scene.model(Sprite::new(&texture), DrawParams::default());
    // let bunny = scene.add_model(&bunny);
    // scene.set_scale(&bunny, vec3(0.1, 1.0, 0.1));
    let mut camera = Camera {
        environment: Environment::SolidColor(WHITE),
        depth_enabled: false,
        projection: Projection::Orthographic,
        position: vec3(0., 10.0, 0.),
        up: vec3(0., -1., 0.),
        target: vec3(0.0, 0., 1.),
        z_near: 0.1,
        z_far: 25.0,
        ..Default::default()
    };

    let mut canvas = ctx.new_canvas();

    loop {
        let t = ctx.time_since_start();
        ctx.clear_screen(GRAY);
        scene.set_rotation(&text, Quat::from_rotation_x(t));
        scene.draw(&mut camera);
        next_frame().await;
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
