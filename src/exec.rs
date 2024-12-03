use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Wake, Waker};

use crate::Error;

// Returns Pending once and finishes immediately once woken.
pub struct FrameFuture {
    pub frame_wakers: Option<Arc<Mutex<Vec<Waker>>>>,
}

impl Future for FrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        if let Some(wakers) = self.frame_wakers.take() {
            wakers.lock().unwrap().push(context.waker().clone());
            eprintln!("frame future pend");
            Poll::Pending
        } else {
            // We were told to step, meaning this future gets destroyed and we run
            // the main future until we call next_frame again and end up in this poll
            // function again.
            eprintln!("frame future ready");
            Poll::Ready(())
        }
    }
}

pub struct FileLoadingFuture {
    pub contents: Arc<Mutex<Option<Result<Vec<u8>, Error>>>>,
}

impl Future for FileLoadingFuture {
    type Output = Result<Vec<u8>, Error>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        let mut contents = self.contents.lock().unwrap();
        if let Some(contents) = contents.take() {
            Poll::Ready(contents)
        } else {
            Poll::Pending
        }
    }
}

fn waker(inner: Arc<Inner<'_>>, id: u64) -> Waker {
    // Cannot use the `Wake` trait, because `Inner` has a lifetime that
    // cannot be used to generate a raw waker.
    unsafe fn clone(data: *const ()) -> RawWaker {
        Arc::increment_strong_count(data.cast::<InnerWaker<'_>>());
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake(data: *const ()) {
        // Even though we generate an unconstrained lifetime here, that's no issue,
        // as we just move data that has a lifetime from one collection to another.
        let data = Arc::<InnerWaker<'_>>::from_raw(data.cast());
        data.wake();
    }
    unsafe fn wake_by_ref(data: *const ()) {
        let data = &*data.cast::<InnerWaker<'_>>();
        data.wake();
    }
    unsafe fn drop(data: *const ()) {
        Arc::<InnerWaker<'_>>::from_raw(data.cast());
    }
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    let raw_waker = RawWaker::new(
        Arc::into_raw(Arc::new(InnerWaker { inner, id })).cast(),
        &VTABLE,
    );
    unsafe { Waker::from_raw(raw_waker) }
}

type LocalFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

/// A simple unoptimized, single threaded executor.
pub struct Executor<'a> {
    inner: Arc<Inner<'a>>,
    pub frame_wakers: Arc<Mutex<Vec<Waker>>>,
}

#[derive(Default)]
struct Inner<'a> {
    // FIXME: use a thread safe queue that doesn't need to lock the read end to
    // push to the write end.
    active_futures: Mutex<VecDeque<LocalFuture<'a>>>,
    waiting_futures: Mutex<HashMap<u64, LocalFuture<'a>>>,
    next_wait_id: AtomicU64,
}

struct InnerWaker<'a> {
    id: u64,
    inner: Arc<Inner<'a>>,
}

impl InnerWaker<'_> {
    fn wake(&self) {
        let future = self
            .inner
            .waiting_futures
            .lock()
            .unwrap()
            .remove(&self.id)
            .expect("already woken");
        self.inner.active_futures.lock().unwrap().push_back(future);
    }
}

impl<'a> Executor<'a> {
    pub fn new(frame_wakers: Arc<Mutex<Vec<Waker>>>) -> Self {
        Self {
            frame_wakers,
            inner: Default::default(),
        }
    }

    /// Returns `true` if all futures have finished.
    pub fn is_empty(&self) -> bool {
        self.inner.active_futures.lock().unwrap().is_empty()
            && self.inner.waiting_futures.lock().unwrap().is_empty()
    }

    /// Runs one future until it returns `Pending`.
    /// Returns `true` if anything was run.
    pub fn tick(&self) -> bool {
        let Some(mut future) = self.inner.active_futures.lock().unwrap().pop_front() else {
            return false;
        };
        let id = self
            .inner
            .next_wait_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let waker = waker(self.inner.clone(), id);
        let mut futures_context = std::task::Context::from_waker(&waker);
        match future.as_mut().poll(&mut futures_context) {
            Poll::Ready(()) => {}
            Poll::Pending => {
                let prev = self
                    .inner
                    .waiting_futures
                    .lock()
                    .unwrap()
                    .insert(id, future);
                assert!(prev.is_none());
            }
        }
        true
    }

    pub fn spawn<Fut: Future<Output = ()> + 'a>(&self, f: Fut) {
        self.inner
            .active_futures
            .lock()
            .unwrap()
            .push_back(Box::pin(f));
    }
}
