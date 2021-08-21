use macroquad::prelude::*;

#[macroquad::main("3D")]
async fn main() {
    let rust_logo = load_texture("examples/rust.png").await.unwrap();
    let ferris = load_texture("examples/ferris.png").await.unwrap();

    let mut cam = vec3(-20., 15., 0.);

    loop {
        clear_background(LIGHTGRAY);

        // Going 3d!

        set_camera(&Camera3D {
            position: cam,
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            ..Default::default()
        });

        draw_grid(20, 1., BLACK, GRAY);

        draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
        draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
        draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);

        draw_plane(vec3(-8., 0., -8.), vec2(5., 5.), ferris, WHITE);

        draw_cube(vec3(-5., 1., -2.), vec3(2., 2., 2.), rust_logo, WHITE);
        draw_cube(vec3(-5., 1., 2.), vec3(2., 2., 2.), ferris, WHITE);
        draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

        draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);

        // Back to screen space, render some text

        set_default_camera();
        draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
        draw_text("Hold arrow keys to rotate the camera", 10.0, 40.0, 30.0, BLACK);
        draw_text(format!("Camera: {:?}", cam).as_str(), 10.0, 60.0, 30.0, BLACK);

        if is_key_down(KeyCode::Left) {
            cam = Mat3::from_rotation_y(-0.05) * cam;
        } else if is_key_down(KeyCode::Right) {
            cam = Mat3::from_rotation_y(0.05) * cam;
        }

        next_frame().await
    }
}
