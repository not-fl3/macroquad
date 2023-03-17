use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(WHITE);
        next_frame().await
    }
}
