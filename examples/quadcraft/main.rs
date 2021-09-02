/*
- Draw Chunk boundaries
- Have toggle labels for stuff
- Toggle camera fly
- Get into own fork
- Clean up examples assets folder
- Make player stand instead of crouch
- Remove all notes from code
*/

mod camera;
mod aabb;
mod physics;
mod world;

use macroquad::prelude::*;
use camera::FirstPersonCamera;
use physics::physics_move;
use world::*;
use aabb::AABB;

// Macroquad max vertices are 8000 and max indices are 4000
pub const CHUNK_SIZE: usize = 8;
pub const VOXEL_SIZE: f32 = 1.0;
pub const VOXEL_HALF: f32 = VOXEL_SIZE / 2.0;
pub const PIXELS_PER_TEXTURE: usize = 16;

#[macroquad::main("Quadcraft")]
async fn main() {
    show_mouse(false);
    set_cursor_grab(true);
    
    let mut camera = FirstPersonCamera::new();
    camera.position = vec3(5.0, 5.0, 5.0);

    let mut game_world = World::new();
    init_world(&mut game_world).await;

    // Player position, velocity, and AABB collision bounds
    let mut origin = vec3(5.0, 5.0, 5.0);
    let mut velocity = Vec3::ZERO;
    let bounds = vec3(0.25, 0.75, 0.25);
    let mut camera_fly = false;
    let mut draw_chunk_boundaries = false;

    loop {
        clear_background(WHITE);

        // Update the camera to allow mouse looking
        camera.update(get_frame_time());

        // Only use custom physics movement, put camera at player position
        // Uncomment this to allow free flying
        if camera_fly {
            // Player follows camera (so it can fall properly when off)
            origin = camera.position;
        }
        else {
            // Camera follows player (so it falls properly when on)
            camera.position = origin;
        }

        // Game Rendering 3D
        set_camera(&camera.get_macroquad_camera());
        draw_grid(20, VOXEL_SIZE, BLACK, BLACK);
        draw_gizmo(vec3(0.0, 0.0, 0.0));

        for chunk in game_world.chunks.values() {
            draw_mesh(&chunk.mesh_opaque);
            draw_mesh(&chunk.mesh_transparent);

            if draw_chunk_boundaries {
                let offset = (
                    Vec3::ONE * (CHUNK_SIZE as f32 / 2.0 - VOXEL_HALF)
                );
                draw_cube_wires(  // Draw the chunk AABB
                    Vec3::ONE * chunk.position.as_f32() + offset,
                    Vec3::ONE * CHUNK_SIZE as f32,
                    Color::new(1.0, 0.216, 0.607, 1.0)
                );
            }
        }

        // Physics Update
        let radius = 2.0;
        let collidables = game_world.get_collidable_blocks(
            AABB::from_box(camera.position, Vec3::ONE * radius)
        );
        let mut block_aabbs = Vec::with_capacity(collidables.len());

        for (aabb, block) in collidables {
            // GREEN: Color::new(155.0 / 255.0, 200.0 / 255.0, 100.0 / 255.0, 1.0)
            // draw_cube_wires(aabb.get_center(), Vec3::ONE, Color::new(1.0, 0.216, 0.607, 1.0));
            block_aabbs.push(aabb);
        }

        physics_move(&mut origin, bounds, &mut velocity, &block_aabbs);

        // Game Rendering 2D
        set_default_camera();

        draw_text(
            format!("{} FPS", get_fps()).as_str(),
            16.0,
            16.0,
            16.0,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );

        // Player input controlls
        const SPEED: f32 = 0.2;
        if is_key_down(KeyCode::Down) {
            velocity += (
                -(camera.front * vec3(1.0, 0.0, 1.0)) *
                SPEED *
                get_frame_time()
            );
        }
        if is_key_down(KeyCode::Up) {
            velocity += (
                (camera.front * vec3(1.0, 0.0, 1.0)) *
                SPEED *
                get_frame_time()
            );
        }
        if is_key_down(KeyCode::Right) {
            velocity += camera.right * SPEED * get_frame_time();
        }
        if is_key_down(KeyCode::Left) {
            velocity += -camera.right * SPEED * get_frame_time();
        }
        if is_key_down(KeyCode::Space) {
            velocity += vec3(0.0, SPEED * 4.0, 0.0) * get_frame_time();
        }
        if is_key_pressed(KeyCode::Escape) { break; }

        next_frame().await
    }
}

pub async fn init_world(world: &mut World) {
    let lab_tile = world.register(
        BlockType {
            texture: String::from("examples/res/res/LabTile.png"),
            opaque: true 
        }
    ).await;

    let water = world.register(
        BlockType {
            texture: String::from("examples/res/res/Water.png"),
            opaque: false
        }
    ).await;

    let lava = world.register(
        BlockType {
            texture: String::from("examples/res/res/Lava.png"),
            opaque: false
        }
    ).await;

    world.register(
        BlockType {
            texture: String::from("examples/res/res/DesertMountain1.png"),
            opaque: true
        }
    ).await;

    let wall = world.register(
        BlockType {
            texture: String::from("examples/res/res/MetalPanel.png"),
            opaque: true
        }
    ).await;

    world.register(
        BlockType {
            texture: String::from("examples/res/res/Sand1.png"),
            opaque: true
        }
    ).await;

    let size = 24i32;
    // for x in -size .. size
    // {
    //     for y in -size .. size
    //     {
    //         for z in -size .. size
    //         {
    //             world.queue_place_block(ivec3(x, y, z), Block { typ: lab_tile });
    //         }
    //     }
    // }

    for x in -size / 4i32 .. (size / 4i32) {
        for z in -size / 4i32 .. (size / 4i32) {
            world.queue_place_block(ivec3(x, 4, z), Block { typ: lab_tile });

            if x == -(size / 4i32) ||
                x == (size / 4i32 - 1i32) ||
                z == -(size / 4i32) ||
                z == (size / 4i32 - 1i32)
            {
                world.queue_place_block(ivec3(x, 5, z), Block { typ: wall });
            }
        }
    }

    for x in -size .. size {
        for z in -size .. size {
            world.queue_place_block(
                ivec3(x, -size - 2, z),
                Block { typ: lab_tile }
            );

            if x == z {
                world.queue_place_block(
                    ivec3(x, -size - 1, z),
                    Block { typ: lava }
                );
            }

            else {
                world.queue_place_block(
                    ivec3(x, -size - 1, z),
                    Block { typ: water }
                );
            }
        }
    }

    world.queue_place_block(ivec3(0, size, 0), Block {typ: water});
    world.rebuild_all();
}

fn draw_gizmo(at: Vec3) {
    draw_line_3d(at, at + vec3(0.0, 0.5, 0.0), GREEN);
    draw_line_3d(at, at + vec3(0.5, 0.0, 0.0), RED);
    draw_line_3d(at, at + vec3(0.0, 0.0, 0.5), BLUE);
}
