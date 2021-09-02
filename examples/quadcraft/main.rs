/*
- Draw Chunk boundaries
- Have toggle labels for stuff
- Toggle camera fly
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
pub const SPEED: f32 = 0.4;

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
    let mut toggle_camera_fly = false;
    let mut toggle_draw_chunk_boundaries = false;
    let mut toggle_draw_block_colliders = false;

    loop {
        clear_background(WHITE);

        // Update the camera to allow mouse looking
        camera.update(get_frame_time());

        // Only use custom physics movement, put camera at player position
        // Uncomment this to allow free flying
        if toggle_camera_fly {
            // Player follows camera (so it can fall properly when off)
            origin = camera.position;
        }
        else {
            // Camera follows player (so it falls properly when on)
            let head_level = origin + vec3(0.0, VOXEL_SIZE, 0.0);
            camera.position = head_level;
        }

        // Game Rendering 3D
        set_camera(&camera.get_macroquad_camera());
        draw_gizmo(vec3(0.0, -10.0, 0.0));

        for chunk in game_world.chunks.values() {
            draw_mesh(&chunk.mesh_opaque);
            draw_mesh(&chunk.mesh_transparent);

            if toggle_draw_chunk_boundaries {
                let offset = 
                    Vec3::ONE * (CHUNK_SIZE as f32 / 2.0 - VOXEL_HALF);
                draw_cube_wires(  // Draw the chunk AABB
                    Vec3::ONE * chunk.position.as_f32() + offset,
                    Vec3::ONE * CHUNK_SIZE as f32,
                    DARKGREEN
                );
            }
        }

        // Physics Update
        let radius = 5.0;
        let collidables = game_world.get_collidable_blocks(
            AABB::from_box(camera.position, Vec3::ONE * radius)
        );
        let mut block_aabbs = Vec::with_capacity(collidables.len());

        for (aabb, _block) in collidables {
            if toggle_draw_block_colliders {
                draw_cube_wires(
                    aabb.get_center(),
                    Vec3::ONE,
                    BLUE
                );
            }
            block_aabbs.push(aabb);
        }

        physics_move(&mut origin, bounds, &mut velocity, &block_aabbs);

        // Game Rendering 2D
        set_default_camera();

        draw_text(
            format!("{} FPS", get_fps()).as_str(),
            16.0,
            32.0 * 1.0,
            32.0,
            BLACK
        );

        draw_text(
            format!("Fly Mode: {}", toggle_camera_fly).as_str(),
            16.0,
            32.0 * 2.0,
            32.0,
            BLACK
        );

        draw_text(
            format!(
                "Show Chunk Bounds: {}",
                toggle_draw_chunk_boundaries
            ).as_str(),
            16.0,
            32.0 * 3.0,
            32.0,
            BLACK
        );

        draw_text(
            format!(
                "Show Block AABBs: {}",
                toggle_draw_block_colliders
            ).as_str(),
            16.0,
            32.0 * 4.0,
            32.0,
            BLACK
        );

        // Player input controlls
        if is_key_down(KeyCode::Down) {
            velocity += 
                -(camera.front * vec3(1.0, 0.0, 1.0)) *
                SPEED *
                get_frame_time();
        }
        if is_key_down(KeyCode::Up) {
            velocity += 
                (camera.front * vec3(1.0, 0.0, 1.0)) *
                SPEED *
                get_frame_time();
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
        if is_key_pressed(KeyCode::Key1) {
            toggle_camera_fly = !toggle_camera_fly;
        }
        if is_key_pressed(KeyCode::Key2) {
            toggle_draw_chunk_boundaries = !toggle_draw_chunk_boundaries;
        }
        if is_key_pressed(KeyCode::Key3) {
            toggle_draw_block_colliders = !toggle_draw_block_colliders;
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await
    }
}

pub async fn init_world(world: &mut World) {
    let lab_tile = world.register(
        BlockType {
            texture: String::from("examples/res/LabTile.png"),
            opaque: true 
        }
    ).await;

    let water = world.register(
        BlockType {
            texture: String::from("examples/res/Water.png"),
            opaque: false
        }
    ).await;

    let lava = world.register(
        BlockType {
            texture: String::from("examples/res/Lava.png"),
            opaque: false
        }
    ).await;

    world.register(
        BlockType {
            texture: String::from("examples/res/DesertMountain1.png"),
            opaque: true
        }
    ).await;

    let wall = world.register(
        BlockType {
            texture: String::from("examples/res/MetalPanel.png"),
            opaque: true
        }
    ).await;

    world.register(
        BlockType {
            texture: String::from("examples/res/Sand1.png"),
            opaque: true
        }
    ).await;

    border_area(
        world,
        ivec3(0, 0, 0),
        ivec3(12, 12, 12),
        Block { typ: lab_tile }
    );

    let size = 24i32;
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

    world.queue_place_block(ivec3(0, -11, 0), Block {typ: lava});
    world.queue_place_block(ivec3(1, -11, 0), Block {typ: water});
    world.queue_place_block(ivec3(-1, -11, 0), Block {typ: water});
    world.queue_place_block(ivec3(0, -11, 1), Block {typ: water});
    world.queue_place_block(ivec3(0, -11, -1), Block {typ: water});
    world.rebuild_all();
}

fn border_area(
    game_world: &mut World,
    position: IVec3,
    volume: IVec3,
    block: Block
)
{
    let aabb = AABB::from_box(position.as_f32(), volume.as_f32());

    let x_min = aabb.min.x.min(aabb.max.x).round() as i32;
    let y_min = aabb.min.y.min(aabb.max.y).round() as i32;
    let z_min = aabb.min.z.min(aabb.max.z).round() as i32;
    let x_max = aabb.max.x.max(aabb.min.x).round() as i32;
    let y_max = aabb.max.y.max(aabb.min.y).round() as i32;
    let z_max = aabb.max.z.max(aabb.min.z).round() as i32;

    for x in x_min ..= x_max {
        for y in y_min ..= y_max {
            for z in z_min ..= z_max {
                if x == x_min ||
                    x == x_max ||
                    y == y_min ||
                    y == y_max ||
                    z == z_min ||
                    z == z_max
                {
                    game_world.queue_place_block(ivec3(x, y, z), block);
                }
            }
        }
    }
}
