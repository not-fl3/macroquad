use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct FrameFuture;
impl Unpin for FrameFuture {}

impl Future for FrameFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let context: &mut i32 = unsafe { std::mem::transmute(context) };

        if *context == 666 {
            *context = 0;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>) {
    let mut futures_context = 666;
    let futures_context_ref: &mut _ = unsafe { std::mem::transmute(&mut futures_context) };
    let _ = future.as_mut().poll(futures_context_ref);
}
