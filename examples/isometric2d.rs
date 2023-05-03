// Example: Isometric 2D
// Shows basic specialized usage of the Camera2D API.

use macroquad::prelude::*;

const TILE_SIZE: IVec2 = ivec2(64, 64);
const MAP_SIZE: IVec2 = ivec2(10, 10);

#[macroquad::main("Isometric 2D")]
async fn main() {
    set_pc_assets_folder("examples");

    let texture = load_texture("grass_v1.png").await.unwrap();
    // Set camera area to some multiple of tile size (in this case 64x64).
    let cam_area = vec2(768., 576.);
    // Assumption here is the world origin is 0, 0.
    let cam_pos = vec2(-cam_area.x / 2., -cam_area.y / 4.);
    let mut camera =
        Camera2D::from_display_rect(Rect::new(cam_pos.x, cam_pos.y, cam_area.x, cam_area.y));

    loop {
        clear_background(BLUE);

        // Exit when `ESC` is pressed.
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Control the camera.
        // W, A, S, D for moving the camera and scroll to zoom.
        let cam_speed = 10.;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            camera.target.y -= cam_speed
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            camera.target.x -= cam_speed;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            camera.target.y += cam_speed;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            camera.target.x += cam_speed;
        }
        let (_, scroll_y) = mouse_wheel();
        camera.zoom *= 1.1_f32.powf(scroll_y);

        // Draw tiles in camera perspective.
        set_camera(&camera);
        for y in 0..MAP_SIZE.y {
            for x in 0..MAP_SIZE.x {
                let world_pos = map_to_world(ivec2(x, y));
                draw_texture(texture, world_pos.x, world_pos.y, WHITE);
            }
        }
        set_default_camera();

        let mouse_in_world = camera.screen_to_world(mouse_position().into());
        draw_text(
            &format!("Pointing at: {}", world_to_map(mouse_in_world)),
            0.,
            20.,
            20.,
            BLACK,
        );

        next_frame().await;
    }
}

// Transform world position to map position.
// Reference: https://youtu.be/04oQ2jOUjkU
fn world_to_map(world_pos: Vec2) -> IVec2 {
    let ihat = vec2(0.5, 0.25) * TILE_SIZE.as_vec2();
    let jhat = vec2(-0.5, 0.25) * TILE_SIZE.as_vec2();
    let inverse = mat2(ihat, jhat).inverse();

    inverse.mul_vec2(world_pos).as_ivec2()
}

// Transform map position to world position.
// Reference: https://youtu.be/04oQ2jOUjkU
fn map_to_world(map_pos: IVec2) -> Vec2 {
    let ihat = vec2(0.5, 0.25) * TILE_SIZE.as_vec2();
    let jhat = vec2(-0.5, 0.25) * TILE_SIZE.as_vec2();
    let transform = mat2(ihat, jhat);
    let offset = ivec2(-TILE_SIZE.x / 2, 0);

    transform.mul_vec2(map_pos.as_vec2()) + offset.as_vec2()
}
