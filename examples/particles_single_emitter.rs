use macroquad::prelude::*;

use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

#[macroquad::main("Fountain")]
async fn main() {
    let mut emitter = Emitter::new(EmitterConfig {
        lifetime: 0.5,
        amount: 5,
        initial_direction_spread: 0.0,
        initial_velocity: -50.0,
        size: 2.0,
        size_curve: Some(Curve {
            points: vec![(0.0, 0.5), (0.5, 1.0), (1.0, 0.0)],
            ..Default::default()
        }),

        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    loop {
        clear_background(BLACK);

        let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 100.0, 100.0));

        set_camera(&camera);

        emitter.draw(vec2(50., 50.));

        next_frame().await
    }
}
