//! Loading and playing sounds.

use crate::{file::load_file, get_context};

use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
#[path = "audio/native_snd.rs"]
mod snd;

#[cfg(target_arch = "wasm32")]
#[path = "audio/web_snd.rs"]
mod snd;

pub struct AudioContext {
    native_ctx: snd::AudioContext,
    sounds: HashMap<usize, snd::Sound>,
    id: usize,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            native_ctx: snd::AudioContext::new(),
            sounds: HashMap::new(),
            id: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound(usize);

/// Load audio file.
///
/// Attempts to automatically detect the format of the source of data.
pub async fn load_sound(path: &str) -> Result<Sound, crate::file::FileError> {
    let data = load_file(path).await?;

    let sound = load_native_snd(&data).await;

    let ctx = &mut get_context().audio_context;
    let id = ctx.id;
    ctx.sounds.insert(id, sound);
    ctx.id += 1;
    Ok(Sound(id))
}

#[cfg(target_arch = "wasm32")]
async fn load_native_snd(data: &[u8]) -> snd::Sound {
    snd::Sound::load(&data).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn load_native_snd(data: &[u8]) -> snd::Sound {
    let ctx = &mut get_context().audio_context.native_ctx;

    snd::Sound::load(ctx, &data)
}
pub fn play_sound_once(sound: Sound) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();

    sound.play(
        &mut ctx.native_ctx,
        PlaySoundParams {
            looped: false,
            volume: 1.0,
        },
    );
}

pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

pub fn play_sound(sound: Sound, params: PlaySoundParams) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();

    sound.play(&mut ctx.native_ctx, params);
}

pub fn stop_sound(sound: Sound) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();

    sound.stop(&mut ctx.native_ctx);
}

pub fn set_sound_volume(sound: Sound, volume: f32) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();
    sound.set_volume(volume)
}
