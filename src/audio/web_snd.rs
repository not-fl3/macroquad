use crate::audio::PlaySoundParams;

extern "C" {
    fn audio_init();
    fn audio_add_buffer(content: *const u8, content_len: u32) -> u32;
    fn audio_play_buffer(buffer: u32, volume_l: f32, volume_r: f32, speed: f32, repeat: bool);
    fn audio_source_is_loaded(buffer: u32) -> bool;
    fn audio_source_set_volume(buffer: u32, volume_l: f32, volume_r: f32);
    fn audio_source_stop(buffer: u32);
}

#[no_mangle]
pub extern "C" fn macroquad_audio_crate_version() -> u32 {
    let major = 0;
    let minor = 1;
    let patch = 0;

    (major << 24) + (minor << 16) + patch
}

pub struct AudioContext;

impl AudioContext {
    pub fn new() -> AudioContext {
        unsafe {
            audio_init();
        }

        AudioContext
    }
}

pub struct Sound(u32);

impl Sound {
    pub async fn load(data: &[u8]) -> Sound {
        use crate::window::next_frame;

        let buffer = unsafe { audio_add_buffer(data.as_ptr(), data.len() as u32) };
        while unsafe { audio_source_is_loaded(buffer) } == false {
            next_frame().await;
        }
        Sound(buffer)
    }

    pub fn play(&mut self, _ctx: &mut AudioContext, params: PlaySoundParams) {
        unsafe { audio_play_buffer(self.0, params.volume, params.volume, 1.0, params.looped) }
    }

    pub fn stop(&mut self, _ctx: &mut AudioContext) {
        unsafe { audio_source_stop(self.0) }
    }

    pub fn set_volume(&mut self, volume: f32) {
        unsafe { audio_source_set_volume(self.0, volume, volume) }
    }
}
