use macroquad::{experimental::coroutines::start_coroutine, window::next_frame};

#[macroquad::test]
async fn coroutine_execution_order() {
    start_coroutine(async move {
        println!("a");
        next_frame().await;
        println!("b");
    });
    println!("c");
    next_frame().await;
    println!("d");
    next_frame().await;
}
