use macroquad::{
    experimental::{coroutines::start_coroutine, scene},
    window::next_frame,
};

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
