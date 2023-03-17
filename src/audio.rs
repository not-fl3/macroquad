//! Loading and playing sounds.

use crate::{file::load_file, get_context, Error};
use std::{collections::HashMap, sync::Arc};

#[cfg(all(feature = "audio"))]
use quad_snd::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(all(feature = "audio"))]
pub use quad_snd::PlaySoundParams;

#[cfg(not(feature = "audio"))]
mod dummy_audio {
    use crate::audio::PlaySoundParams;

    pub struct AudioContext {}

    impl AudioContext {
        pub fn new() -> AudioContext {
            AudioContext {}
        }

        #[cfg(target_os = "android")]
        pub fn pause(&mut self) {}

        #[cfg(target_os = "android")]
        pub fn resume(&mut self) {}
    }

    pub struct Sound {}

    impl Sound {
        pub fn load(_ctx: &mut AudioContext, _data: &[u8]) -> Sound {
            Sound {}
        }

        pub fn play(&self, _ctx: &mut AudioContext, _params: PlaySoundParams) {
            eprintln!("warn: macroquad's \"audio\" feature disabled.");
        }

        pub fn stop(&self, _ctx: &mut AudioContext) {}

        pub fn set_volume(&self, _ctx: &mut AudioContext, _volume: f32) {}

        pub fn is_loaded(&self) -> bool {
            false
        }

        pub fn delete(&self, _ctx: &AudioContext) {}
    }
}

#[cfg(not(feature = "audio"))]
use dummy_audio::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(not(feature = "audio"))]
pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

pub struct AudioContext {
    native_ctx: QuadSndContext,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            native_ctx: QuadSndContext::new(),
        }
    }

    #[cfg(target_os = "android")]
    pub fn pause(&mut self) {
        self.native_ctx.pause()
    }

    #[cfg(target_os = "android")]
    pub fn resume(&mut self) {
        self.native_ctx.resume()
    }
}

struct QuadSndSoundGuarded(QuadSndSound);

impl Drop for QuadSndSoundGuarded {
    fn drop(&mut self) {
        let ctx = &get_context().audio_context;
        self.0.delete(&ctx.native_ctx);
    }
}

#[derive(Clone)]
pub struct Sound(Arc<QuadSndSound>);

impl std::fmt::Debug for Sound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sound").finish()
    }
}

/// Load audio file.
///
/// Attempts to automatically detect the format of the source of data.
pub async fn load_sound(path: &str) -> Result<Sound, Error> {
    let data = load_file(path).await?;

    load_sound_from_bytes(&data).await
}

/// Load audio data.
///
/// Attempts to automatically detect the format of the source of data.
pub async fn load_sound_from_bytes(data: &[u8]) -> Result<Sound, Error> {
    let sound = {
        let ctx = &mut get_context().audio_context;
        QuadSndSound::load(&mut ctx.native_ctx, data)
    };

    // only on wasm the sound is not ready right away
    #[cfg(target_arch = "wasm32")]
    while sound.is_loaded() == false {
        crate::window::next_frame().await;
    }

    let ctx = &mut get_context().audio_context;

    Ok(Sound(Arc::new(sound)))
}

pub fn play_sound_once(sound: &Sound) {
    let ctx = &mut get_context().audio_context;

    sound.0.play(
        &mut ctx.native_ctx,
        PlaySoundParams {
            looped: false,
            volume: 1.0,
        },
    );
}

pub fn play_sound(sound: &Sound, params: PlaySoundParams) {
    let ctx = &mut get_context().audio_context;
    sound.0.play(&mut ctx.native_ctx, params);
}

pub fn stop_sound(sound: &Sound) {
    let ctx = &mut get_context().audio_context;
    sound.0.stop(&mut ctx.native_ctx);
}

pub fn set_sound_volume(sound: &Sound, volume: f32) {
    let ctx = &mut get_context().audio_context;
    sound.0.set_volume(&mut ctx.native_ctx, volume)
}
