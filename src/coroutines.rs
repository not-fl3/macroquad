use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::get_context;

#[derive(Debug, PartialEq)]
enum ExecState {
    RunOnce,
    Waiting,
}

pub(crate) struct FutureContext {
    state: ExecState,
}

pub(crate) struct CoroutinesContext {
    futures: Vec<Option<(Pin<Box<dyn Future<Output = ()>>>, FutureContext)>>,
}

impl CoroutinesContext {
    pub fn new() -> CoroutinesContext {
        CoroutinesContext { futures: vec![] }
    }

    pub fn update(&mut self) {
        for future in &mut self.futures {
            if let Some((f, context)) = future {
                let futures_context_ref: &mut _ = unsafe { std::mem::transmute(context) };

                if matches!(f.as_mut().poll(futures_context_ref), Poll::Ready(_)) {
                    *future = None;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coroutine {
    id: usize,
}

pub unsafe fn start_coroutine(future: impl Future<Output = ()>) -> Coroutine {
    let context = &mut get_context().coroutines_context;

    let boxed_future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(future);
    let boxed_future = std::mem::transmute(boxed_future);

    context.futures.push(Some((
        boxed_future,
        FutureContext {
            state: ExecState::RunOnce,
        },
    )));

    Coroutine {
        id: context.futures.len() - 1,
    }
}

pub unsafe fn stop_all_coroutines() {
   let context = &mut get_context().coroutines_context;

    context.futures.clear();
}

pub fn stop_coroutine(coroutine: Coroutine) {
    let context = &mut get_context().coroutines_context;

    context.futures[coroutine.id] = None;
}

pub struct TimerDelayFuture {
    pub(crate) start_time: f64,
    pub(crate) time: f32,
}
impl Unpin for TimerDelayFuture {}

impl Future for TimerDelayFuture {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        if miniquad::date::now() - self.start_time >= self.time as f64 {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}

pub fn wait_seconds(time: f32) -> TimerDelayFuture {
    TimerDelayFuture {
        start_time: miniquad::date::now(),
        time,
    }
}

pub mod tweens {
    use std::future::Future;
    use std::pin::Pin;
    use std::{ops::{Sub, Add, Mul}, task::{Context, Poll}};

    pub struct LinearTweanFuture<T> where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> {
        var: *mut T,
        from: T,
        to: T,
        start_time: f64,
        time: f32,
    }
    impl<T> Unpin for LinearTweanFuture<T> where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> {}

    impl<T> Future for LinearTweanFuture<T> where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> {
        type Output = Option<()>;

        fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
            let t = (miniquad::date::now() - self.start_time) / self.time as f64;
            if t <= 1. {
                unsafe { *self.var = self.from + (self.to - self.from) * t as f32 };
                Poll::Pending
            } else {
                unsafe { *self.var = self.to };
                Poll::Ready(Some(()))
            }
        }
    }

    pub fn linear<T>(x: &mut T, to: T, time: f32) -> LinearTweanFuture<T> where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> {
        LinearTweanFuture {
            var: x as *mut _,
            to,
            from: *x,
            time,
            start_time: miniquad::date::now()
        }
    }
}
