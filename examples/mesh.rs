use std::f32::consts::TAU;
use macroquad::prelude::*;
use macroquad::models::Vertex;

fn create_disk_mesh(position: Vec3, radius: f32, sides: usize, texture: Option<Texture2D>, color: Color) -> Mesh {

    // Start with adding the center vertex in the center of the disk.
    let mut vertices = vec![Vertex {
        position: Vec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        },
        uv: Vec2 { x: 0.5, y: 0.5 },
        color,
    }];

    // Add vertices on the edge of the face. The disk is on the x,z plane. Y is up.
    for i in 0..sides {
        let angle = TAU * (i as f32) / sides as f32;
        let (sin, cos) = angle.sin_cos();
        // uv's are in percentages of the texture
        let u = 0.5 + 0.5 * cos;
        let v = 0.5 + 0.5 * sin;
        vertices.push(Vertex {
            position: Vec3 {
                x: position.x + radius * cos,
                y: position.y,
                z: position.z + radius * sin,
            },
            uv: Vec2 { x: u, y: v },
            color,
        });
    }

    // Tie three vertices together at a time to form triangle indices.
    let num_vertices = vertices.len();
    let mut indices: Vec<u16> = Vec::new();

    for i in 1..num_vertices - 1 {
        indices.push(0);
        indices.push(i as u16);
        indices.push((i + 1) as u16);
    }

    indices.push(0);
    indices.push(1);
    indices.push((num_vertices - 1) as u16);

    Mesh {
        vertices,
        indices,
        texture,
    }
}

fn create_tube_mesh(position: Vec3, height: f32, radius: f32, sides: usize, texture: Option<Texture2D>, color: Color) -> Mesh {
    let mut vertices: Vec<Vertex> = Vec::new();

    // Top ring of vertices. Add 1 to sides to close the loop.
    // Set uv's to wrap texture around the tube
    for i in 0..sides + 1 {
        let angle = (i as f32) * TAU / sides as f32;
        let (sin, cos) = angle.sin_cos();
        // uv's are percentages of the texture size
        let u = 1.0 - 1.0 / sides as f32 * i as f32;
        vertices.push(Vertex {
            position: Vec3 {
                x: position.x + radius * cos,
                y: position.y,
                z: position.z + radius * sin,
            },
            uv: Vec2 { x: u, y: 1.0 },
            color,
        });
    }

    // Bottom ring of vertices
    for i in 0..sides + 1 {
        let angle = (i as f32) * TAU / sides as f32;
        let (sin, cos) = angle.sin_cos();
        // uv's are percentages of the texture size
        let u = 1.0 - 1.0 / sides as f32 * i as f32;
        vertices.push(Vertex {
            position: Vec3 {
                x: position.x + radius * cos,
                y: position.y + height,
                z: position.z + radius * sin,
            },
            uv: Vec2 { x: u, y: 0.0 },
            color,
        });
    }

    let mut indices: Vec<u16> = Vec::new();

    // Each side is a quad which is two triangles
    for i in 0..sides {
        indices.push(i as u16);
        indices.push((i + 1) as u16);
        indices.push((i + sides + 1) as u16);

        indices.push((i + 1) as u16);
        indices.push((i + sides + 1) as u16);
        indices.push((i + sides + 2) as u16);
    }

    Mesh {
        vertices,
        indices,
        texture,
    }
}

/// Mesh example
/// Create three meshes to form a cylinder. Two disks for top and bottom and
/// a tube for the cylinder sides. Apply texture to the disks and the tube.
/// Move camera around the mesh by rotating the camera position around the x axis and
/// then around the y axis using the arrow keys. Escape key to exit.
#[macroquad::main("Mesh")]
async fn main() {
    let rust_logo = load_texture("examples/rust.png").await.unwrap();

    let mut camera_angle_x: f32 = 45.0;
    let mut camera_angle_y: f32 = 0.0;
    let mut camera_position = Vec3::new(0.0, 0.0, 0.0);
    let camera_distance: f32 = 90.0;

    // Top of cylinder
    let top_disk = create_disk_mesh(
        vec3(0.0, 60.0, 0.0),
        10.0,
        20,
        Some(rust_logo.clone()),
        WHITE);

    // Bottom of cylinder
    let bottom_disk = create_disk_mesh(
        vec3(0.0, 0.0, 0.0),
        10.0,
        20,
        Some(rust_logo.clone()),
        WHITE);

    // Tube - cylinder wall
    let tube = create_tube_mesh(
        vec3(0.0, 0.0, 0.0),
        60.0,
        10.0,
        20,
        Some(rust_logo.clone()),
        WHITE);

    loop {
        clear_background(LIGHTGRAY);

        camera_angle_x = camera_angle_x % 360.0;
        camera_angle_x = if camera_angle_x < 0.0 { camera_angle_x + 360.0 } else { camera_angle_x };
        let up = if (camera_angle_x < 90.0) | (camera_angle_x >= 270.0) { 1.0 } else { -1.0 };

        let (a_sin, a_cos) = camera_angle_x.to_radians().sin_cos();
        let (b_sin, b_cos) = camera_angle_y.to_radians().sin_cos();
        let x_z_radius = camera_distance * a_cos;

        camera_position.y = camera_distance * a_sin + 30.0;
        camera_position.x = x_z_radius * b_cos;
        camera_position.z = x_z_radius * b_sin;

        set_camera(&Camera3D {
            position: camera_position,
            up: vec3(0., up, 0.),
            target: vec3(0., 30., 0.),
            ..Default::default()
        });

        draw_mesh(&top_disk);
        draw_mesh(&bottom_disk);
        draw_mesh(&tube);

        next_frame().await;

        if is_key_down(KeyCode::Escape) {
            break;
        }
        if is_key_down(KeyCode::Right) {
            camera_angle_y += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            camera_angle_y -= 1.0;
        }
        if is_key_down(KeyCode::Up) {
            camera_angle_x += 1.0;
        }
        if is_key_down(KeyCode::Down) {
            camera_angle_x -= 1.0;
        }
    }
}