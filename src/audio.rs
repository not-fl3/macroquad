//! Loading and playing sounds.

#[cfg(feature="audio")]
use crate::{file::load_file, get_context};
#[cfg(feature="audio")]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature="audio")]
#[path = "audio/native_snd.rs"]
mod snd;

#[cfg(target_arch = "wasm32")]
#[cfg(feature="audio")]
#[path = "audio/web_snd.rs"]
mod snd;

#[cfg(feature="audio")]
pub struct AudioContext {
    native_ctx: snd::AudioContext,
    sounds: HashMap<usize, snd::Sound>,
    id: usize,
}

#[cfg(feature="audio")]
impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            native_ctx: snd::AudioContext::new(),
            sounds: HashMap::new(),
            id: 0,
        }
    }
}

#[cfg(feature="audio")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound(usize);

/// Load audio file.
///
/// Attempts to automatically detect the format of the source of data.
#[cfg(feature="audio")]
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
#[cfg(feature="audio")]
async fn load_native_snd(data: &[u8]) -> snd::Sound {
    snd::Sound::load(&data).await
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature="audio")]
async fn load_native_snd(data: &[u8]) -> snd::Sound {
    let ctx = &mut get_context().audio_context.native_ctx;

    snd::Sound::load(ctx, &data)
}
#[cfg(feature="audio")]
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

#[cfg(feature="audio")]
pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

#[cfg(feature="audio")]
pub fn play_sound(sound: Sound, params: PlaySoundParams) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();

    sound.play(&mut ctx.native_ctx, params);
}

#[cfg(feature="audio")]
pub fn stop_sound(sound: Sound) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();

    sound.stop(&mut ctx.native_ctx);
}

#[cfg(feature="audio")]
pub fn set_sound_volume(sound: Sound, volume: f32) {
    let ctx = &mut get_context().audio_context;
    let sound = &mut ctx.sounds.get_mut(&sound.0).unwrap();
    sound.set_volume(volume)
}
