use macroquad::prelude::*;

use macroquad_tiled as tiled;

use macroquad_platformer::*;

struct Player {
    collider: Actor,
    speed: Vec2,
}

struct Platform {
    collider: Solid,
    speed: f32,
}

#[macroquad::main("Platformer")]
async fn main() {
    let tileset = load_texture("examples/tileset.png").await.unwrap();
    tileset.set_filter(FilterMode::Nearest);

    let tiled_map_json = load_string("examples/map.json").await.unwrap();
    let tiled_map = tiled::load_map(&tiled_map_json, &[("tileset.png", tileset)], &[]).unwrap();

    let mut static_colliders = vec![];
    for (_x, _y, tile) in tiled_map.tiles("main layer", None) {
        static_colliders.push(if tile.is_some() {
            Tile::Solid
        } else {
            Tile::Empty
        });
    }

    let mut world = World::new();
    world.add_static_tiled_layer(static_colliders, 8., 8., 40, 1);

    let mut player = Player {
        collider: world.add_actor(vec2(50.0, 80.0), 8, 8),
        speed: vec2(0., 0.),
    };

    let mut platform = Platform {
        collider: world.add_solid(vec2(170.0, 130.0), 32, 8),
        speed: 50.,
    };

    let camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, 152.0));

    loop {
        clear_background(BLACK);

        set_camera(&camera);

        tiled_map.draw_tiles("main layer", Rect::new(0.0, 0.0, 320.0, 152.0), None);

        // draw platform
        {
            let pos = world.solid_pos(platform.collider);
            tiled_map.spr_ex(
                "tileset",
                Rect::new(6.0 * 8.0, 0.0, 32.0, 8.0),
                Rect::new(pos.x, pos.y, 32.0, 8.0),
            )
        }

        // draw player
        {
            // sprite id from tiled
            const PLAYER_SPRITE: u32 = 120;

            let pos = world.actor_pos(player.collider);
            if player.speed.x >= 0.0 {
                tiled_map.spr("tileset", PLAYER_SPRITE, Rect::new(pos.x, pos.y, 8.0, 8.0));
            } else {
                tiled_map.spr(
                    "tileset",
                    PLAYER_SPRITE,
                    Rect::new(pos.x + 8.0, pos.y, -8.0, 8.0),
                );
            }
        }

        // player movement control
        {
            let pos = world.actor_pos(player.collider);
            let on_ground = world.collide_check(player.collider, pos + vec2(0., 1.));

            if on_ground == false {
                player.speed.y += 500. * get_frame_time();
            }

            if is_key_down(KeyCode::Right) {
                player.speed.x = 100.0;
            } else if is_key_down(KeyCode::Left) {
                player.speed.x = -100.0;
            } else {
                player.speed.x = 0.;
            }

            if is_key_pressed(KeyCode::Space) {
                if on_ground {
                    player.speed.y = -120.;
                }
            }

            world.move_h(player.collider, player.speed.x * get_frame_time());
            world.move_v(player.collider, player.speed.y * get_frame_time());
        }

        // platform movement
        {
            world.solid_move(platform.collider, platform.speed * get_frame_time(), 0.0);
            let pos = world.solid_pos(platform.collider);
            if platform.speed > 1. && pos.x >= 220. {
                platform.speed *= -1.;
            }
            if platform.speed < -1. && pos.x <= 150. {
                platform.speed *= -1.;
            }
        }

        next_frame().await
    }
}
