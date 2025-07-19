use macroquad::prelude::*;
use std::fs::read;

fn conf() -> Conf {
    let image =
        Image::from_file_with_format(&read("examples/rustacean_happy.png").unwrap(), None).unwrap();
    let icon = image.as_app_icon();

    Conf {
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    loop {
        clear_background(LIGHTGRAY);
        next_frame().await;
    }
}
