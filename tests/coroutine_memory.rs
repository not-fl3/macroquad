use macroquad::{experimental::coroutines::start_coroutine, telemetry, window::next_frame};

#[macroquad::test]
async fn coroutine_memory() {
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
