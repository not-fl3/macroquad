use macroquad::prelude::*;
// use glam::vec3;

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;


fn conf() -> Conf
{
    Conf {
        window_title: String::from("Macroquad"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    set_cursor_grab(true);
    show_mouse(false);

    let mut x = 0.0;
    let mut switch = false;
    let bounds = 8.0;

    let world_up = vec3(0.0, 1.0, 0.0);
    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;

    let mut front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos()
    ).normalize();
    let mut right = front.cross(world_up).normalize();
    let mut up;

    let mut position = vec3(0.0, 1.0, 0.0);
    let mut last_mouse_position: Vec2 = mouse_position().into();

    loop {
        let delta = get_frame_time();
        
        if is_key_pressed(KeyCode::Escape) { break; }

        if is_key_down(KeyCode::Up) { position += front * MOVE_SPEED; }
        if is_key_down(KeyCode::Down) { position -= front * MOVE_SPEED; }
        if is_key_down(KeyCode::Left) { position -= right * MOVE_SPEED; }
        if is_key_down(KeyCode::Right) { position += right * MOVE_SPEED; }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;
        last_mouse_position = mouse_position;

        yaw += mouse_delta.x * delta * LOOK_SPEED;
        pitch += mouse_delta.y * delta * -LOOK_SPEED;

        pitch = if pitch > 1.5 { 1.5 } else { pitch };
        pitch = if pitch < -1.5 { -1.5 } else { pitch };
    
        front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos()
        ).normalize();

        right = front.cross(world_up).normalize();
        up = right.cross(front).normalize();


        x += if switch { 0.04 } else { -0.04 };
        if x >= bounds || x <= -bounds { switch = !switch; }

        clear_background(Color::new(1.0, 0.7, 0.0, 1.0));

        // Going 3d!

        set_camera(Camera3D {
            position: position,
            up: up,
            target: position + front,
            ..Default::default()
        });

        draw_grid(20, 1.);

        draw_line_3d(vec3(x, 0.0, x), vec3(5.0, 5.0, 5.0), Color::new(1.0, 1.0, 0.0, 1.0));

        draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
        draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
        draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

        // Back to screen space, render some text

        set_default_camera();
        draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);

        draw_text(format!("X: {} Y: {}", mouse_position.x, mouse_position.y).as_str(), 10.0, 48.0 + 18.0, 30.0, BLACK);

        next_frame().await
    }
}
