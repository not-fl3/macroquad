use macroquad::prelude::{coroutines::start_coroutine, next_frame};

#[macroquad::main("back to the future")]
async fn main() {
    start_coroutine(async {
        next_frame().await;
        next_frame().await;
    });
    for _ in 0..10 {
        next_frame().await;
    }
}
