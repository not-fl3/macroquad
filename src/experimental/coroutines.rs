//! The way to emulate multitasking with macroquad's `.await`.
//! Useful for organizing state machines, animation cutscenes and other stuff that require
//! some evaluation over time.
//!

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{exec::ExecState, get_context};

pub(crate) struct CoroutinesContext {
    futures: Vec<Option<(Pin<Box<dyn Future<Output = ()>>>, ExecState)>>,
}

impl CoroutinesContext {
    pub fn new() -> CoroutinesContext {
        CoroutinesContext {
            futures: Vec::with_capacity(1000),
        }
    }

    pub fn update(&mut self) {
        for future in &mut self.futures {
            if let Some((f, context)) = future {
                *context = ExecState::RunOnce;

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

impl Coroutine {
    pub fn is_done(&self) -> bool {
        let context = &get_context().coroutines_context;

        context.futures[self.id].is_none()
    }
}

pub fn start_coroutine(future: impl Future<Output = ()> + 'static + Send) -> Coroutine {
    let context = &mut get_context().coroutines_context;

    let boxed_future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(future);
    let boxed_future = unsafe { std::mem::transmute(boxed_future) };

    context
        .futures
        .push(Some((boxed_future, ExecState::RunOnce)));

    Coroutine {
        id: context.futures.len() - 1,
    }
}

pub fn stop_all_coroutines() {
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

/// Special built-in coroutines for modifying values over time.
pub mod tweens {
    use crate::experimental::scene::{Handle, Lens, Node};
    use std::future::Future;
    use std::pin::Pin;
    use std::{
        ops::{Add, Mul, Sub},
        task::{Context, Poll},
    };

    pub struct LinearTweenFuture<T>
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
    {
        from: T,
        to: T,
        lens: Lens<T>,
        start_time: f64,
        time: f32,
    }
    impl<T> Unpin for LinearTweenFuture<T> where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>
    {
    }

    impl<T> Future for LinearTweenFuture<T>
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
    {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
            let t = (miniquad::date::now() - self.start_time) / self.time as f64;
            let this = self.get_mut();
            let var = this.lens.get();

            // node with value was deleted
            if var.is_none() {
                return Poll::Ready(());
            }
            let var = var.unwrap();

            if t <= 1. {
                *var = this.from + (this.to - this.from) * t as f32;

                Poll::Pending
            } else {
                *var = this.to;

                Poll::Ready(())
            }
        }
    }

    pub fn linear<T, T1, F>(
        handle: Handle<T1>,
        lens: F,
        from: T,
        to: T,
        time: f32,
    ) -> LinearTweenFuture<T>
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
        T1: Node,
        F: for<'r> FnMut(&'r mut T1) -> &'r mut T,
    {
        LinearTweenFuture {
            to,
            from,
            lens: handle.lens(lens),
            time,
            start_time: miniquad::date::now(),
        }
    }

    pub async fn follow_path<T, T1, F>(handle: Handle<T1>, mut lens: F, path: Vec<T>, time: f32)
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
        T1: Node,
        F: for<'r> FnMut(&'r mut T1) -> &'r mut T,
    {
        for point in path.windows(2) {
            linear(
                handle,
                &mut lens,
                point[0],
                point[1],
                time / path.len() as f32,
            )
            .await
        }
    }
}
