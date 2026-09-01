use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
};

#[macroquad::main("Audio")]
async fn main() {
    set_pc_assets_folder("examples");

    let _coroutine = coroutines::start_coroutine(async move {
        let sound_handle = load_sound("sound.wav").await;

        //to simulate it taking a while to load this file, lets do this in a nice loop :)

        for v in 0..10000 {
            load_sound("sound.wav").await.unwrap();
            if v % 10 == 0 {
                println!("Loaded the file {v} times")
            }
        }

        println!("Finished loading sound. Playing...");
        match sound_handle {
            Ok(music) => play_sound(
                &music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            ),
            Err(e) => eprintln!("Failed to load epic music :( -- {e}"),
        }
    });
    let mut x = 0.;
    let speed = 20.;
    loop {
        x += speed * get_frame_time();
        draw_rectangle(x, 10., 20., 20., GREEN);
        next_frame().await;
    }
}
