use macroquad::{experimental::coroutines::start_coroutine, telemetry, window::next_frame};

#[macroquad::test]
async fn coroutine_value() {
    let mut coroutine = start_coroutine(async move {
        next_frame().await;
        1
    });

    coroutine.set_manual_poll();

    assert_eq!(coroutine.retrieve(), None);

    coroutine.poll(0.0);
    coroutine.poll(0.0);

    assert_eq!(coroutine.retrieve(), Some(1));
}

#[macroquad::test]
async fn coroutine_memory() {
    use macroquad::prelude::*;

    for _ in 0..20 {
        start_coroutine(async move {
            next_frame().await;
        });

        next_frame().await;
    }

    // wait for the last one to finish
    next_frame().await;

    assert_eq!(telemetry::active_coroutines_count(), 0);
}
