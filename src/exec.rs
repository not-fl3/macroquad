use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// Returns Pending as long as its inner bool is false.
#[derive(Default)]
pub struct FrameFuture {
    done: bool,
}

impl Future for FrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        if self.done {
            // We were told to step, meaning this future gets destroyed and we run
            // the main future until we call next_frame again and end up in this poll
            // function again.
            Poll::Ready(())
        } else {
            self.done = true;
            Poll::Pending
        }
    }
}

type FileResult<T> = Result<T, crate::file::FileError>;

pub struct FileLoadingFuture {
    pub contents: std::rc::Rc<std::cell::RefCell<Option<FileResult<Vec<u8>>>>>,
}

impl Future for FileLoadingFuture {
    type Output = FileResult<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        let mut contents = self.contents.borrow_mut();
        if let Some(contents) = contents.take() {
            Poll::Ready(contents)
        } else {
            Poll::Pending
        }
    }
}

fn waker() -> Waker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake(_data: *const ()) {
        panic!(
            "macroquad does not support waking futures, please use coroutines, \
            otherwise your pending future will block until the next frame"
        )
    }
    unsafe fn wake_by_ref(data: *const ()) {
        wake(data)
    }
    unsafe fn drop(_data: *const ()) {
        // Nothing to do
    }
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

/// returns true if future is done, false if it would block
pub(crate) fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>) -> bool {
    let waker = waker();
    let mut futures_context = std::task::Context::from_waker(&waker);
    matches!(future.as_mut().poll(&mut futures_context), Poll::Ready(()))
}
