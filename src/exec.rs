use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::prelude::coroutines::{step_coroutine, Coroutine};
use crate::MAIN_FUTURE;

// "resume" sets this to true once SEEN_FRAME is true
pub(crate) static NEXT_FRAME: AtomicBool = AtomicBool::new(false);

// Returns Pending as long as its inner bool is false.
#[derive(Default)]
pub struct FrameFuture {
    done: bool,
}
impl Unpin for FrameFuture {}

impl Future for FrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        if self.done && NEXT_FRAME.load(Ordering::Relaxed) {
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
    pub contents: std::rc::Rc<std::cell::RefCell<(Option<Waker>, Option<FileResult<Vec<u8>>>)>>,
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
        let mut contents = self.contents.borrow_mut();
        if let Some(contents) = contents.1.take() {
            Poll::Ready(contents)
        } else {
            contents.0 = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

pub fn waker(coroutine: Option<Coroutine>) -> Waker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake(data: *const ()) {
        // SAFETY: the only place a data field has been set is in the transmute
        // further down, and we only transmute from an Option<Coroutine>.
        let coroutine: Option<Coroutine> = std::mem::transmute(data);
        if let Some(coroutine) = coroutine {
            step_coroutine(coroutine);
        } else {
            if let Some(future) = MAIN_FUTURE.as_mut() {
                resume(future, false);
            }
        }
    }
    unsafe fn wake_by_ref(data: *const ()) {
        wake(data)
    }
    unsafe fn drop(_data: *const ()) {
        // Nothing to do
    }
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    // SAFETY: transmute ensures that the sizes match up. This transmute to a raw pointer
    // is safe because there is nothing acting on its value except the other functions declared
    // in this function.
    let data = unsafe { std::mem::transmute(coroutine) };
    let raw_waker = RawWaker::new(data, &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

/// returns true if future is done
pub fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>, over_frame: bool) -> bool {
    NEXT_FRAME.store(over_frame, Ordering::Relaxed);
    let waker = waker(None);
    let mut futures_context = std::task::Context::from_waker(&waker);
    matches!(future.as_mut().poll(&mut futures_context), Poll::Ready(_))
}
