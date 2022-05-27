//! The way to emulate multitasking with macroquad's `.await`.
//! Useful for organizing state machines, animation cutscenes and other stuff that require
//! some evaluation over time.
//!

use std::any::{Any, TypeId};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::exec::resume;
use crate::get_context;

mod generational_storage;

use generational_storage::{GenerationalId, GenerationalStorage};

struct CoroutineInternal {
    future: Pin<Box<dyn Future<Output = Box<dyn Any>>>>,
    manual_poll: bool,
    manual_time: Option<f64>,
    // if return value of a coroutine is () there is no need to
    // keep coroutine's memory allocated until the user retrieves the data
    // we can free the memory right away, and just return () on retrieve
    has_value: bool,
}

enum CoroutineState {
    Running(CoroutineInternal),
    Value(Box<dyn Any>),
    Nothing,
}

impl CoroutineState {
    pub fn is_value(&self) -> bool {
        matches!(self, CoroutineState::Value(_))
    }

    pub fn is_nothing(&self) -> bool {
        matches!(self, CoroutineState::Nothing)
    }

    pub fn take_value(&mut self) -> Option<Box<dyn Any>> {
        if self.is_value() {
            let state = std::mem::replace(self, CoroutineState::Nothing);
            if let CoroutineState::Value(v) = state {
                return Some(v);
            }
        }

        None
    }
}

pub(crate) struct CoroutinesContext {
    coroutines: GenerationalStorage<CoroutineState>,
    active_coroutine_now: Option<f64>,
    active_coroutine_delta: Option<f64>,
}

impl CoroutinesContext {
    pub fn new() -> CoroutinesContext {
        CoroutinesContext {
            coroutines: GenerationalStorage::new(),
            active_coroutine_now: None,
            active_coroutine_delta: None,
        }
    }

    pub fn update(&mut self) {
        self.coroutines.retain(|coroutine| {
            if let CoroutineState::Running(ref mut f) = coroutine {
                if f.manual_poll == false {
                    if let Some(v) = resume(&mut f.future) {
                        if f.has_value {
                            *coroutine = CoroutineState::Value(v);
                        } else {
                            return false;
                        }
                    }
                }
            }

            true
        });
    }

    pub(crate) fn allocated_memory(&self) -> usize {
        self.coroutines.allocated_memory()
    }

    pub(crate) fn active_coroutines_count(&self) -> usize {
        self.coroutines.count()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Coroutine<T = ()> {
    id: GenerationalId,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Any> Coroutine<T> {
    /// Returns true if the coroutine finished or was stopped.
    pub fn is_done(&self) -> bool {
        let context = &get_context().coroutines_context;

        let coroutine = context.coroutines.get(self.id);

        if let Some(coroutine) = coroutine {
            return coroutine.is_value() || coroutine.is_nothing();
        }

        return true;
    }

    pub fn retrieve(&self) -> Option<T> {
        let context = &mut get_context().coroutines_context;

        // () is a special case. Futures with () as a return type do not keep
        // their state after finish, so just return ()
        if self.is_done() && TypeId::of::<()>() == TypeId::of::<T>() {
            // well, I wish tehre was a better way to do this..
            let res = Box::new(()) as Box<dyn Any>;
            return Some(*res.downcast().unwrap());
        }

        let coroutine = context.coroutines.get_mut(self.id);
        if let Some(v) = coroutine.and_then(|c| c.take_value()) {
            let res = Some(*v.downcast().unwrap());
            context.coroutines.free(self.id);
            return res;
        }

        None
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

        if let Some(CoroutineState::Running(coroutine)) = context.coroutines.get_mut(self.id) {
            coroutine.manual_time = Some(0.);
            coroutine.manual_poll = true;
        }
    }

    /// Poll coroutine once and advance coroutine's timeline by `delta_time`
    /// Things like `wait_for_seconds` will wait for time in this local timeline`
    /// Will panic if coroutine.manual_poll == false
    pub fn poll(&mut self, delta_time: f64) {
        let context = &mut get_context().coroutines_context;

        let coroutine = context.coroutines.get_mut(self.id);

        // coroutine was finished already
        if coroutine.is_none() {
            return;
        }

        let coroutine = coroutine.unwrap();
        if let CoroutineState::Running(f) = coroutine {
            context.active_coroutine_now = f.manual_time;
            context.active_coroutine_delta = Some(delta_time);
            *f.manual_time.as_mut().unwrap() += delta_time;
            if let Some(v) = resume(&mut f.future) {
                if f.has_value {
                    *coroutine = CoroutineState::Value(v);
                } else {
                    context.coroutines.free(self.id);
                }
            }
            context.active_coroutine_now = None;
            context.active_coroutine_delta = None;
        }
    }
}

pub fn start_coroutine<T: 'static + Any>(
    future: impl Future<Output = T> + 'static + Send,
) -> Coroutine<T> {
    let context = &mut get_context().coroutines_context;

    let has_value = TypeId::of::<()>() != TypeId::of::<T>();

    let id = context
        .coroutines
        .push(CoroutineState::Running(CoroutineInternal {
            future: Box::pin(async { Box::new(future.await) as _ }),
            has_value,
            manual_poll: false,
            manual_time: None,
        }));

    Coroutine {
        id,
        _phantom: PhantomData,
    }
}

pub fn stop_all_coroutines() {
    let context = &mut get_context().coroutines_context;

    context.coroutines.clear();
}

pub fn stop_coroutine(coroutine: Coroutine) {
    let context = &mut get_context().coroutines_context;

    context.coroutines.free(coroutine.id);
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
