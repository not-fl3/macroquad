use macroquad::prelude::*;

use macroquad::experimental::{
    coroutines::{start_coroutine, Coroutine, UnsafeCoroutineState},
    scene::{self, Handle, Node, RefMut},
};

use macroquad::ui::{hash, root_ui};

struct Player {
    pos: Vec2,
    size: Vec2,
    move_coroutine: Option<Coroutine>,
}

#[derive(PartialEq)]
struct PlayerState {
    pos: Vec2,
    size: Vec2,
    coroutine_state: Option<UnsafeCoroutineState>,
}

/// A nice, non-linear movement animation
async fn pulse_move(player: Handle<Player>, dir: Vec2) {
    let steps = 15;
    let speed = 0.01;

    for i in 0..steps {
        scene::get_node(player).pos +=
            vec2((i * i * i) as f32 * speed, (i * i * i) as f32 * speed) * dir;
        scene::get_node(player).size = vec2(100.0 + i as f32 * 2., 100.0 + i as f32 * 2.);
        next_frame().await;
    }
    for i in (0..steps).rev() {
        scene::get_node(player).pos +=
            vec2((i * i * i) as f32 * speed, (i * i * i) as f32 * speed) * dir;
        scene::get_node(player).size = vec2(100.0 + i as f32 * 2., 100.0 + i as f32 * 2.);
        next_frame().await;
    }
}

impl Player {
    fn do_async_thing(&mut self, thing: impl std::future::Future<Output = ()> + Send + 'static) {
        if self.move_coroutine.is_none() {
            let mut coroutine = start_coroutine(thing);
            coroutine.set_manual_poll();
            self.move_coroutine = Some(coroutine);
        }
    }

    fn save_state(&self) -> PlayerState {
        PlayerState {
            pos: self.pos,
            size: self.size,
            coroutine_state: self.move_coroutine.map(|coroutine| coroutine.save_state()),
        }
    }

    fn restore_state(&mut self, state: &PlayerState) {
        self.pos = state.pos;
        self.size = state.size;
        self.move_coroutine = unsafe { state.coroutine_state.as_ref().map(|x| x.recover()) }
    }

    fn tick(mut node: RefMut<Player>) {
        let handle = node.handle();

        if is_key_down(KeyCode::Right) {
            node.do_async_thing(pulse_move(handle, vec2(1., 0.)));
        } else if is_key_down(KeyCode::Left) {
            node.do_async_thing(pulse_move(handle, vec2(-1., 0.)));
        } else if is_key_down(KeyCode::Up) {
            node.do_async_thing(pulse_move(handle, vec2(0., -1.)));
        } else if is_key_down(KeyCode::Down) {
            node.do_async_thing(pulse_move(handle, vec2(0., 1.)));
        }

        if let Some(mut coroutine) = node.move_coroutine {
            if !coroutine.is_done() {
                drop(node);
                coroutine.poll(0.1)
            } else {
                node.move_coroutine = None;
            }
        }
    }
}

impl Node for Player {
    fn draw(node: RefMut<Player>) {
        draw_rectangle(
            node.pos.x - (node.size.x - 100.) / 2.,
            node.pos.y - (node.size.y - 100.) / 2.,
            node.size.x,
            node.size.y,
            RED,
        );
    }
}

#[macroquad::main("Rollback")]
async fn main() {
    let player = scene::add_node(Player {
        pos: vec2(100., 100.),
        size: vec2(100., 100.),
        move_coroutine: None,
    });

    let mut frames = vec![];

    let mut frame = 0.;
    let mut pause = false;

    loop {
        clear_background(WHITE);

        scene::set_camera(
            0,
            Some(Camera2D::from_display_rect(Rect::new(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
            ))),
        );

        let state = scene::get_node(player).save_state();

        if !pause {
            let player = scene::get_node(player);
            Player::tick(player);

            if frame != frames.len() as _ {
                frames.truncate(frame as usize);
            }

            if frames.last().map_or(true, |frame| *frame != state) {
                frames.push(state);
            }
            frame = frames.len() as _;
        }

        if pause && frame as usize > 1 {
            scene::get_node(player).restore_state(&frames[frame as usize - 1]);
        }

        if is_key_pressed(KeyCode::Space) {
            pause ^= true;
        }

        root_ui().slider(
            hash!(),
            &format!("frames: {}", frames.len()),
            0.0..frames.len() as f32,
            &mut frame,
        );
        if pause {
            root_ui().label(None, "Paused, \"Space\" to resume");
        } else {
            root_ui().label(None, "Unpaused, \"Space\" to pause");
        }

        next_frame().await
    }
}
