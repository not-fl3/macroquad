use std::future::Future;
use std::pin::Pin;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// Used to detect "bad" futures being used
#[cfg(debug_assertions)]
static NEXT_FRAME: AtomicBool = AtomicBool::new(false);

// Returns Pending as long as its inner bool is false.
#[derive(Default)]
pub struct FrameFuture {
    done: bool,
}
impl Unpin for FrameFuture {}

impl Future for FrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        if self.done {
            // We were told to step, meaning this future gets destroyed and we run
            // the main future until we call next_frame again and end up in this poll
            // function again.
            Poll::Ready(())
        } else {
            #[cfg(debug_assertions)]
            NEXT_FRAME.store(true, Ordering::Relaxed);
            self.done = true;
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

    fn poll(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        let mut contents = self.contents.borrow_mut();
        if let Some(contents) = contents.take() {
            Poll::Ready(contents)
        } else {
            #[cfg(debug_assertions)]
            NEXT_FRAME.store(true, Ordering::Relaxed);
            Poll::Pending
        }
    }
}

pub fn waker() -> Waker {
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

/// returns true if future is done
pub fn resume(future: &mut Pin<Box<dyn Future<Output = ()>>>) -> bool {
    #[cfg(debug_assertions)]
    NEXT_FRAME.store(false, Ordering::Relaxed);
    let waker = waker();
    let mut futures_context = std::task::Context::from_waker(&waker);
    match future.as_mut().poll(&mut futures_context) {
        Poll::Ready(()) => true,
        Poll::Pending => {
            #[cfg(debug_assertions)]
            assert!(
                NEXT_FRAME.load(Ordering::Relaxed),
                "your macroquad main loop blocked on a future other than a file load or next_frame"
            );
            false
        }
    }
}
