//! Loading and playing sounds.

#![allow(dead_code)]

use crate::{file::load_file, Error};
use std::{cell::RefCell, sync::Arc};

#[cfg(feature = "audio")]
use quad_snd::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(feature = "audio")]
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

        #[allow(dead_code)]
        pub fn is_loaded(&self) -> bool {
            true
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
        with_audio_context(|ctx| {
            self.0.delete(ctx);
        });
    }
}

#[derive(Clone)]
pub struct Sound(Arc<QuadSndSoundGuarded>);

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
    let sound = with_audio_context(|ctx| QuadSndSound::load(ctx, data));

    // only on wasm the sound is not ready right away
    #[cfg(target_arch = "wasm32")]
    while sound.is_loaded() == false {
        crate::window::next_frame().await;
    }

    Ok(Sound(Arc::new(QuadSndSoundGuarded(sound))))
}

thread_local! {
    static AUDIO_CONTEXT: RefCell<Option<QuadSndContext>> = RefCell::new(None);
}

pub(crate) fn init_sound() {
    AUDIO_CONTEXT.with_borrow_mut(|opt| *opt = Some(QuadSndContext::new()));
}

fn with_audio_context<R, F>(f: F) -> R
where
    F: FnOnce(&mut QuadSndContext) -> R,
{
    AUDIO_CONTEXT.with_borrow_mut(|opt| {
        let ctx = opt
            .as_mut()
            .expect("the macroquad audiocontext is not initialized on current thread");
        f(ctx)
    })
}

pub fn play_sound_once(sound: &Sound) {
    with_audio_context(|ctx| {
        sound.0 .0.play(
            ctx,
            PlaySoundParams {
                looped: false,
                volume: 1.0,
            },
        );
    });
}

pub fn play_sound(sound: &Sound, params: PlaySoundParams) {
    with_audio_context(|ctx| {
        sound.0 .0.play(ctx, params);
    });
}

pub fn stop_sound(sound: &Sound) {
    with_audio_context(|ctx| {
        sound.0 .0.stop(ctx);
    });
}

pub fn set_sound_volume(sound: &Sound, volume: f32) {
    with_audio_context(|ctx| {
        sound.0 .0.set_volume(ctx, volume);
    });
}
