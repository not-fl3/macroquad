use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
        fullscreen: false,
        icon: Some(Icon {
            small: include_bytes!("./ico16.png"),
            medium: include_bytes!("./ico32.png"),
            big: include_bytes!("./ico64.png"),
        }),
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
