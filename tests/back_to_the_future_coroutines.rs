use std::sync::{Arc, Mutex};

use macroquad::prelude::{
    coroutines::{start_coroutine, wait_seconds},
    next_frame,
};

#[macroquad::test]
async fn back_to_the_future_coroutine() {
    struct Player {
        on_ground: bool,
        allow_movement: bool,
    }
    let player = Arc::new(Mutex::new(Player {
        on_ground: false,
        allow_movement: false,
    }));
    let player2 = player.clone();
    start_coroutine(async move {
        loop {
            if player.lock().unwrap().on_ground {
                break;
            }
            next_frame().await;
        }
        println!("before wait");
        wait_seconds(1.0).await;
        println!("after wait");
        player.lock().unwrap().allow_movement = true;
    });
    let mut i = 10;
    loop {
        println!("{}", i);
        if player2.lock().unwrap().allow_movement {
            break;
        }
        if i == 0 {
            player2.lock().unwrap().on_ground = true;
        }
        i -= 1;
        next_frame().await;
    }
    assert!(i < -1, "coroutine blocked main thread");
}
