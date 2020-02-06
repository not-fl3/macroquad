use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, PartialEq)]
pub enum ExecState {
    RunOnce,
    Waiting,
}

pub struct FrameFuture;
impl Unpin for FrameFuture {}

impl Future for FrameFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let context: &mut ExecState = unsafe { std::mem::transmute(context) };

        if *context == ExecState::RunOnce {
            *context = ExecState::Waiting;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub struct TextureLoadingFuture {
    pub texture: std::rc::Rc<std::cell::RefCell<Option<crate::drawing::Texture2D>>>,
}
impl Unpin for TextureLoadingFuture {}

impl Future for TextureLoadingFuture {
    type Output = crate::drawing::Texture2D;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let context: &mut ExecState = unsafe { std::mem::transmute(context) };

        if *context == ExecState::Waiting {
            Poll::Pending
        } else if let Some(texture) = self.texture.borrow_mut().take() {
            *context = ExecState::Waiting;
            Poll::Ready(texture)
        } else {
            Poll::Pending
        }
    }
}

pub fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>) {
    let mut futures_context = ExecState::RunOnce;
    let futures_context_ref: &mut _ = unsafe { std::mem::transmute(&mut futures_context) };
    let _ = future.as_mut().poll(futures_context_ref);
}
