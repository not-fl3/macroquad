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

type FileResult<T> = Result<T, crate::file::FileError>;

pub struct FileLoadingFuture {
    pub contents: std::rc::Rc<std::cell::RefCell<Option<FileResult<Vec<u8>>>>>,
}
// TODO: use mutex(?) instead of refcell here
// this is still safe tho - macroquad's executor is refcell-safe
// but this just look too bad
unsafe impl Send for FileLoadingFuture {}
unsafe impl Sync for FileLoadingFuture {}

impl Unpin for FileLoadingFuture {}
impl Future for FileLoadingFuture {
    type Output = FileResult<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let context: &mut ExecState = unsafe { std::mem::transmute(context) };

        if *context == ExecState::Waiting {
            Poll::Pending
        } else if let Some(contents) = self.contents.borrow_mut().take() {
            *context = ExecState::Waiting;
            Poll::Ready(contents)
        } else {
            Poll::Pending
        }
    }
}

/// returns true if future is done
pub fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>) -> bool {
    let mut futures_context = ExecState::RunOnce;
    let futures_context_ref: &mut _ = unsafe { std::mem::transmute(&mut futures_context) };

    matches!(future.as_mut().poll(futures_context_ref), Poll::Ready(_))
}
