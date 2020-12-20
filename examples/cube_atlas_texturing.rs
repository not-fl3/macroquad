use macroquad::prelude::*;

#[macroquad::main("CubeAtlasTexturing")]
async fn main() {
    let cube_letters_tileset = load_texture("examples/cube_letters_tileset.png").await;
    let step = 0.2;
    let mut x = -5.;
    let mut y = 5.;

    loop {
        clear_background(WHITE);

        if is_key_down(KeyCode::Right) && x >= -10. && x < (10. - step) {
            x += step;
        }
        if is_key_down(KeyCode::Left) && x > (-10. + step) && x < 10. {
            x -= step;
        }
        if is_key_down(KeyCode::Down) && y > (-10. + step) && y < 10. {
            y -= step;
        }
        if is_key_down(KeyCode::Up) && y >= -10. && y < (10. - step) {
            y += step;
        }

        set_camera(Camera3D {
            position: vec3(x, y, 10.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            ..Default::default()
        });

        draw_grid(20, 1.);

        draw_cube_ex(
            vec3(0., 0., 0.),
            vec3(5., 5., 5.),
            // In this example, the same texture atlas is reused but multiple
            // assets can be used.
            [
                cube_letters_tileset,
                cube_letters_tileset,
                cube_letters_tileset,
                cube_letters_tileset,
                cube_letters_tileset,
                cube_letters_tileset,
            ],
            [
                // Front face.
                QuadTextureParams {
                    source: Some(Rect::new(0., 0., 100., 100.)),
                    rotation: Some(Turn::Half),
                },
                // Back face.
                QuadTextureParams {
                    source: Some(Rect::new(100., 0., 100., 100.)),
                    rotation: Some(Turn::Half),
                },
                // Top face.
                QuadTextureParams {
                    source: Some(Rect::new(0., 100., 100., 100.)),
                    rotation: Some(Turn::ThreeQuarter),
                },
                // Bottom face.
                QuadTextureParams {
                    source: Some(Rect::new(100., 100., 100., 100.)),
                    rotation: Some(Turn::ThreeQuarter),
                },
                // Right face.
                QuadTextureParams {
                    source: Some(Rect::new(0., 200., 100., 100.)),
                    rotation: Some(Turn::ThreeQuarter),
                },
                // Left face.
                QuadTextureParams {
                    source: Some(Rect::new(100., 200., 100., 100.)),
                    rotation: Some(Turn::Quarter),
                },
            ],
            WHITE,
        );

        set_default_camera();

        draw_text(
            "Use the arrow keys to move the camera",
            10.0,
            20.0,
            30.0,
            BLACK,
        );

        next_frame().await
    }
}
