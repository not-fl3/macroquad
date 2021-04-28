use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::audio::PlaySoundParams;

pub struct AudioContext {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        let (_stream, handle) = OutputStream::try_default().unwrap();

        AudioContext { _stream, handle }
    }
}

pub struct Sound {
    // the only way to play the same sound looped or once in rodio :/
    sink_once: Sink,
    sink_looped: Sink,
    looped: bool,
    // and also rodio cant really reset sound, so we need to recreate it after each stop :/
    source: Vec<u8>,
}

impl Sound {
    pub fn load(ctx: &mut AudioContext, data: &[u8]) -> Sound {
        let sink_once = Sink::try_new(&ctx.handle).unwrap();
        let sink_looped = Sink::try_new(&ctx.handle).unwrap();
        sink_once.pause();
        sink_looped.pause();

        let decoder = rodio::Decoder::new(std::io::Cursor::new(data.to_vec())).unwrap();
        sink_once.append(decoder);

        let decoder = rodio::Decoder::new_looped(std::io::Cursor::new(data.to_vec())).unwrap();
        sink_looped.append(decoder);

        Sound {
            sink_once,
            sink_looped,
            source: data.to_vec(),
            looped: false,
        }
    }

    pub fn play(&mut self, ctx: &mut AudioContext, params: PlaySoundParams) {
        self.stop(ctx);

        self.looped = params.looped;

        if self.looped {
            self.sink_looped.set_volume(params.volume);
            self.sink_looped.play();
        } else {
            self.sink_once.set_volume(params.volume);
            self.sink_once.play();
        }
    }

    pub fn stop(&mut self, ctx: &mut AudioContext) {
        self.sink_once.stop();
        self.sink_looped.stop();

        let sink_once = Sink::try_new(&ctx.handle).unwrap();
        let sink_looped = Sink::try_new(&ctx.handle).unwrap();
        sink_once.pause();
        sink_looped.pause();

        let decoder = rodio::Decoder::new(std::io::Cursor::new(self.source.clone())).unwrap();
        sink_once.append(decoder);

        let decoder =
            rodio::Decoder::new_looped(std::io::Cursor::new(self.source.clone())).unwrap();
        sink_looped.append(decoder);

        self.sink_once = sink_once;
        self.sink_looped = sink_looped;
    }

    pub fn set_volume(&mut self, volume: f32) {
        if self.looped {
            self.sink_looped.set_volume(volume);
        } else {
            self.sink_once.set_volume(volume);
        }
    }
}
