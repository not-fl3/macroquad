use macroquad::{
    experimental::{
        coroutines::{start_coroutine, wait_seconds},
        scene,
    },
    window::next_frame,
};

#[macroquad::test]
async fn coroutine_execution_order() {
    start_coroutine(async move {
        println!("a");
        next_frame().await;
        println!("b");
    });
    println!("c");
    next_frame().await;
    println!("d");
    next_frame().await;
}

#[macroquad::test]
async fn coroutine_manual_poll() {
    struct Player {
        state: i32,
    }
    impl scene::Node for Player {}

    let player = scene::add_node(Player { state: 0 });

    let mut coroutine = start_coroutine(async move {
        loop {
            scene::get_node(player).state += 1;
            next_frame().await;
        }
    });

    // make sure that coroutine is not yet polled
    assert_eq!(scene::get_node(player).state, 0);

    coroutine.set_manual_poll();

    // still not polled
    assert_eq!(scene::get_node(player).state, 0);

    coroutine.poll(0.1);
    assert_eq!(scene::get_node(player).state, 1);

    next_frame().await;
    next_frame().await;

    // make sure that after main loop's next_frame coroutine was not polled
    assert_eq!(scene::get_node(player).state, 1);

    // and that we still can poll
    coroutine.poll(0.1);
    assert_eq!(scene::get_node(player).state, 2);
}

#[macroquad::test]
async fn coroutine_manual_poll_delay() {
    struct Player {
        state: i32,
    }
    impl scene::Node for Player {}

    let player = scene::add_node(Player { state: 0 });

    let mut coroutine = start_coroutine(async move {
        wait_seconds(1.).await;
        scene::get_node(player).state = 1;
    });

    coroutine.set_manual_poll();

    assert_eq!(scene::get_node(player).state, 0);

    // not 1 second yet, coroutine will have "now": 0.0, "delta": 0.9, (0.0 + 0.9) < 1.0
    coroutine.poll(0.9);

    assert_eq!(scene::get_node(player).state, 0);

    // coroutine will have "now": 0.1, delta: 0.11, (0.9 + 0.11) > 1.0, wait_for_seconds pass
    coroutine.poll(0.11);

    assert_eq!(scene::get_node(player).state, 1);
}

#[macroquad::test]
async fn coroutine_restore() {
    struct Player {
        state: i32,
    }
    impl scene::Node for Player {}

    let player = scene::add_node(Player { state: 0 });

    let mut coroutine = start_coroutine(async move {
        scene::get_node(player).state += 1;
        next_frame().await;
        scene::get_node(player).state += 1;
        next_frame().await;
        scene::get_node(player).state += 1;
        next_frame().await;
        scene::get_node(player).state += 1;
        next_frame().await;
        scene::get_node(player).state += 1;
    });

    coroutine.set_manual_poll();

    for i in 0..5 {
        assert_eq!(scene::get_node(player).state, i);
        coroutine.poll(0.1);
    }

    // restore state
    scene::get_node(player).state = 2;

    for i in 0..5 {
        assert_eq!(scene::get_node(player).state, i);
        coroutine.poll(0.1);
    }
}

#[test]
fn raw_coroutine() {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Poll, RawWaker, RawWakerVTable, Waker};

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

    struct Kaboom {
        counter: i32,
    }

    impl Future for Kaboom {
        type Output = ();

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Self::Output> {
            println!("{}", self.counter);
            self.as_mut().counter -= 1;

            if self.counter == 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    let future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(Kaboom { counter: 5 });
    let wtf: [u64; 2] = unsafe { std::mem::transmute(future) };

    let mut future: Pin<Box<dyn Future<Output = ()>>> = unsafe { std::mem::transmute(wtf) };

    let waker = waker();
    let mut futures_context = std::task::Context::from_waker(&waker);
    let _ = future.as_mut().poll(&mut futures_context);
    let _ = future.as_mut().poll(&mut futures_context);

    let mut future: Pin<Box<dyn Future<Output = ()>>> = unsafe { std::mem::transmute(wtf) };

    let _ = future.as_mut().poll(&mut futures_context);
    let _ = future.as_mut().poll(&mut futures_context);
}
