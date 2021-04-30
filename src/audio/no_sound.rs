use crate::audio::PlaySoundParams;

pub struct AudioContext {}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {}
    }
}

pub struct Sound {}

impl Sound {
    pub fn load(_ctx: &mut AudioContext, _data: &[u8]) -> Sound {
        Sound {}
    }

    pub fn play(&mut self, _ctx: &mut AudioContext, _params: PlaySoundParams) {}

    pub fn stop(&mut self, _ctx: &mut AudioContext) {}

    pub fn set_volume(&mut self, _volume: f32) {}
}
