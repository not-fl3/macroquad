//! The way to emulate multitasking with macroquad's `.await`.
//! Useful for organizing state machines, animation cutscenes and other stuff that require
//! some evaluation over time.
//!

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::exec::resume;
use crate::get_context;

struct CoroutineInternal {
    future: Pin<Box<dyn Future<Output = ()>>>,
    manual_poll: bool,
    manual_time: Option<f64>,
}

pub(crate) struct CoroutinesContext {
    coroutines: Vec<Option<CoroutineInternal>>,
    active_coroutine_now: Option<f64>,
    active_coroutine_delta: Option<f64>,
}

impl CoroutinesContext {
    pub fn new() -> CoroutinesContext {
        CoroutinesContext {
            coroutines: Vec::with_capacity(1000),
            active_coroutine_now: None,
            active_coroutine_delta: None,
        }
    }

    pub fn update(&mut self) {
        for future in &mut self.coroutines {
            if let Some(f) = future {
                if f.manual_poll == false && resume(&mut f.future) {
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

        context.coroutines[self.id].is_none()
    }

    /// By default coroutines are being polled each frame, inside the "next_frame()"
    ///
    /// ```skip
    /// start_coroutine(async move {
    ///    println!("a");
    ///    next_frame().await;
    ///    println!("b");
    /// }); // <- coroutine is created, but not yet polled
    /// println!("c"); // <- first print, "c"
    /// next_frame().await; // coroutine will be polled for the first time
    ///                     // will print "a"
    /// println!("d");      // "d"
    /// next_frame().await; // coroutine will be polled second time, pass next_frame().await and will print "b"
    /// ```
    /// will print "cadb" (there is a test for it, "tests/coroutine.rs:coroutine_execution_order" )
    ///
    /// But, sometimes, automatic polling is not nice
    /// good example - game pause. Imagine a player that have some "update" function
    /// and some coroutines runned. During the pause "update" just early quit, but
    /// what with the coroutines?
    ///
    /// "set_manual_poll" allows to control how coroutine is beng polled
    /// after set_manual_poll() coroutine will never be polled automatically
    /// so player will need to poll all its coroutines inside "update" function
    pub fn set_manual_poll(&mut self) {
        let context = &mut get_context().coroutines_context;

        if let Some(coroutine) = &mut context.coroutines[self.id] {
            coroutine.manual_time = Some(0.);
            coroutine.manual_poll = true;
        }
    }

    /// Poll coroutine once and advance coroutine's timeline by `delta_time`
    /// Things like `wait_for_seconds` will wait for time in this local timeline`
    /// Will panic if coroutine.manual_poll == false
    pub fn poll(&mut self, delta_time: f64) {
        let context = &mut get_context().coroutines_context;

        let coroutine = &mut context.coroutines[self.id];
        if let Some(f) = coroutine {
            context.active_coroutine_now = f.manual_time;
            context.active_coroutine_delta = Some(delta_time);
            *f.manual_time.as_mut().unwrap() += delta_time;
            if resume(&mut f.future) {
                *coroutine = None;
            }
            context.active_coroutine_now = None;
            context.active_coroutine_delta = None;
        }
    }
}

pub fn start_coroutine(future: impl Future<Output = ()> + 'static + Send) -> Coroutine {
    let context = &mut get_context().coroutines_context;

    let boxed_future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(future);

    context.coroutines.push(Some(CoroutineInternal {
        future: boxed_future,
        manual_poll: false,
        manual_time: None,
    }));

    Coroutine {
        id: context.coroutines.len() - 1,
    }
}

pub fn stop_all_coroutines() {
    let context = &mut get_context().coroutines_context;

    // Cannot clear the vector as there may still be outstanding Coroutines
    // so their ids would now point into nothingness or later point into
    // different Coroutines.
    for future in &mut context.coroutines {
        *future = None;
    }
}

pub fn stop_coroutine(coroutine: Coroutine) {
    let context = &mut get_context().coroutines_context;

    context.coroutines[coroutine.id] = None;
}

pub struct TimerDelayFuture {
    pub(crate) remaining_time: f32,
}

impl Future for TimerDelayFuture {
    type Output = Option<()>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        let delta = get_context()
            .coroutines_context
            .active_coroutine_delta
            .unwrap_or(crate::time::get_frame_time() as _);

        self.remaining_time -= delta as f32;

        if self.remaining_time <= 0.0 {
            Poll::Ready(Some(()))
        } else {
            Poll::Pending
        }
    }
}

pub fn wait_seconds(time: f32) -> TimerDelayFuture {
    TimerDelayFuture {
        remaining_time: time,
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
