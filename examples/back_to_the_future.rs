use std::{future::Future, task::Poll};

#[macroquad::main("back to the future")]
async fn main() {
    struct Kaboom;
    impl Future for Kaboom {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> Poll<Self::Output> {
            let cloned = cx.waker().clone(); // segmentation fault
            drop(cloned);
            Poll::Ready(())
        }
    }
    Kaboom.await;
}
