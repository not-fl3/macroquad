use macroquad::{audio, prelude::*, ui};

#[macroquad::main("Audio")]
async fn main() {
    set_pc_assets_folder("examples");

    let sound1 = audio::load_sound("sound.wav").await.unwrap();
    let sound2 = audio::load_sound("sound2.wav").await.unwrap();

    loop {
        clear_background(LIGHTGRAY);

        if ui::root_ui().button(None, "Play sound 1") {
            warn!("play 1!");
            audio::play_sound_once(&sound1);
        }
        if ui::root_ui().button(None, "Play sound 2") {
            warn!("play 2!");
            audio::play_sound_once(&sound2);
        }
        next_frame().await
    }
}
