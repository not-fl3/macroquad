// and a little bit of glue code to make it work with macroquad

use crate::{audio::PlaySoundParams, logging::info};

use std::sync::mpsc;

// Slightly reduced OpenSLES implementation
// from an amazing "audir" library: https://github.com/norse-rs/audir/
mod opensles {
    use audir_sles as sles;
    use std::os::raw::c_void;
    use std::ptr;

    use super::info;

    pub mod api {
        pub mod channel_mask {
            pub const FRONT_LEFT: u32 = 0b0001;
            pub const FRONT_RIGHT: u32 = 0b0010;
            pub const FRONT_CENTER: u32 = 0b0100;
        }

        pub type ChannelMask = u32;

        #[allow(dead_code)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum Format {
            F32,
            U32,
        }

        /// Sample description.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct SampleDesc {
            /// Sample Format.
            pub format: Format,
            /// Sample Rate.
            pub sample_rate: usize,
        }

        /// Frame description.
        ///
        /// Consists of a channel mask and a sample description.
        /// A frame is composed of one samples per channel.
        #[derive(Debug, Copy, Clone)]
        pub struct FrameDesc {
            /// Sample Format.
            pub format: Format,
            /// Sample Rate.
            pub sample_rate: usize,
            /// Channel Mask.
            pub channels: ChannelMask,
        }

        #[derive(Debug, Clone)]
        pub struct DeviceDesc {
            pub sample_desc: SampleDesc,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Channels {
            pub input: ChannelMask,
            pub output: ChannelMask,
        }

        /// Device Stream properties.
        #[derive(Debug, Clone, Copy)]
        pub struct StreamProperties {
            pub channels: ChannelMask,
            pub sample_rate: usize,
            pub buffer_size: usize,
        }

        impl StreamProperties {
            pub fn num_channels(&self) -> usize {
                ((self.channels & channel_mask::FRONT_LEFT) != 0) as usize
                    + ((self.channels & channel_mask::FRONT_RIGHT) != 0) as usize
                    + ((self.channels & channel_mask::FRONT_CENTER) != 0) as usize
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct StreamBuffers {
            /// Number of frames per buffer.
            pub frames: usize,

            /// Input frame buffer.
            ///
            /// For streams with empty input channels the pointer will be null.
            /// The buffer pointer is aligned according to the stream format requirements.
            pub input: *const (),

            /// Input frame buffer.
            ///
            /// For streams with empty output channels the pointer will be null.
            /// The buffer pointer is aligned according to the stream format requirements.
            pub output: *mut (),
        }

        pub struct Stream {
            pub properties: StreamProperties,
            pub buffers: StreamBuffers,
        }

        pub type StreamCallback = Box<dyn FnMut(Stream) + Send>;
    }

    const BUFFER_NUM_FRAMES: usize = 1024; // TODO: random
    const BUFFER_CHAIN_SIZE: usize = 3; // TODO

    fn map_channel_mask(mask: api::ChannelMask) -> sles::SLuint32 {
        let mut channels = 0;
        if mask & api::channel_mask::FRONT_LEFT != 0 {
            channels |= sles::SL_SPEAKER_FRONT_LEFT;
        }
        if mask & api::channel_mask::FRONT_RIGHT != 0 {
            channels |= sles::SL_SPEAKER_FRONT_RIGHT;
        }
        channels
    }

    struct CallbackData {
        buffers: Vec<Vec<u32>>,
        cur_buffer: usize,
        callback: api::StreamCallback,
        frame_desc: api::FrameDesc,
    }

    pub struct Instance {
        _instance: sles::SLObjectItf,
        engine: sles::SLEngineItf,
    }

    impl Instance {
        pub unsafe fn new() -> Self {
            let mut instance = ptr::null();
            let result = sles::slCreateEngine(
                &mut instance,
                0,
                ptr::null(),
                0,
                ptr::null(),
                ptr::null_mut(),
            );
            info!("sles::slCreateEngine: {:?}", result);

            let result = ((**instance).Realize).unwrap()(instance, sles::SL_BOOLEAN_FALSE as _);
            info!("sles::instance::Realise: {:?}", result);

            let mut engine = ptr::null();
            ((**instance).GetInterface).unwrap()(
                instance,
                sles::SL_IID_ENGINE,
                &mut engine as *mut _ as _,
            );

            Instance {
                engine,
                _instance: instance,
            }
        }

        pub unsafe fn create_device(
            &self,
            desc: api::DeviceDesc,
            channels: api::Channels,
            callback: api::StreamCallback,
        ) -> Result<Device, ()> {
            let mut mix = ptr::null();
            ((**self.engine).CreateOutputMix).unwrap()(
                self.engine,
                &mut mix,
                0,
                ptr::null(),
                ptr::null(),
            );
            ((**mix).Realize).unwrap()(mix, sles::SL_BOOLEAN_FALSE as _);

            let mut audio_player = ptr::null();
            let mut locator_source = sles::SLDataLocator_AndroidSimpleBufferQueue {
                locatorType: sles::SL_DATALOCATOR_ANDROIDSIMPLEBUFFERQUEUE as _,
                numBuffers: BUFFER_CHAIN_SIZE as _,
            };

            let mut create_player = |format| {
                let mut source = sles::SLDataSource {
                    pLocator: &mut locator_source as *mut _ as _,
                    pFormat: format,
                };
                let mut locator_sink = sles::SLDataLocator_OutputMix {
                    locatorType: sles::SL_DATALOCATOR_OUTPUTMIX as _,
                    outputMix: mix,
                };
                let mut sink = sles::SLDataSink {
                    pLocator: &mut locator_sink as *mut _ as _,
                    pFormat: ptr::null_mut(),
                };
                let ids = [sles::SL_IID_BUFFERQUEUE];
                let requirements = [sles::SL_BOOLEAN_TRUE];

                let result = ((**self.engine).CreateAudioPlayer).unwrap()(
                    self.engine,
                    &mut audio_player,
                    &mut source,
                    &mut sink,
                    1,
                    ids.as_ptr(),
                    requirements.as_ptr() as _,
                );
                info!("sles::engine::CreateAudioPlayer: {}", result);
            };

            let sles_channels = map_channel_mask(channels.output);
            let num_channels = sles_channels.count_ones();

            match desc.sample_desc.format {
                api::Format::F32 => {
                    let mut format_source = sles::SLAndroidDataFormat_PCM_EX {
                        formatType: sles::SL_ANDROID_DATAFORMAT_PCM_EX as _,
                        numChannels: num_channels as _,
                        sampleRate: (desc.sample_desc.sample_rate * 1000) as _,
                        bitsPerSample: sles::SL_PCMSAMPLEFORMAT_FIXED_32 as _,
                        containerSize: sles::SL_PCMSAMPLEFORMAT_FIXED_32 as _,
                        channelMask: sles_channels,
                        endianness: sles::SL_BYTEORDER_LITTLEENDIAN as _, // TODO
                        representation: sles::SL_ANDROID_PCM_REPRESENTATION_FLOAT as _,
                    };

                    create_player(&mut format_source as *mut _ as _);
                }
                api::Format::U32 => {
                    let mut format_source = sles::SLDataFormat_PCM {
                        formatType: sles::SL_DATAFORMAT_PCM as _,
                        numChannels: num_channels as _,
                        samplesPerSec: (desc.sample_desc.sample_rate * 1000) as _,
                        bitsPerSample: sles::SL_PCMSAMPLEFORMAT_FIXED_32 as _,
                        containerSize: sles::SL_PCMSAMPLEFORMAT_FIXED_32 as _,
                        channelMask: sles_channels,
                        endianness: sles::SL_BYTEORDER_LITTLEENDIAN as _, // TODO
                    };

                    create_player(&mut format_source as *mut _ as _);
                }
            }

            ((**audio_player).Realize).unwrap()(audio_player, sles::SL_BOOLEAN_FALSE as _);

            let mut queue: sles::SLAndroidSimpleBufferQueueItf = ptr::null();
            ((**audio_player).GetInterface).unwrap()(
                audio_player,
                sles::SL_IID_BUFFERQUEUE,
                &mut queue as *mut _ as _,
            );

            let mut state: sles::SLPlayItf = ptr::null();
            ((**audio_player).GetInterface).unwrap()(
                audio_player,
                sles::SL_IID_PLAY,
                &mut state as *mut _ as _,
            );

            let buffers = (0..BUFFER_CHAIN_SIZE)
                .map(|_| {
                    let buffer_size = num_channels as usize * BUFFER_NUM_FRAMES;
                    let mut buffer = Vec::<u32>::with_capacity(buffer_size);
                    buffer.set_len(buffer_size);
                    buffer
                })
                .collect();

            let frame_desc = api::FrameDesc {
                format: desc.sample_desc.format,
                channels: channels.output,
                sample_rate: desc.sample_desc.sample_rate,
            };

            let data = Box::new(CallbackData {
                buffers,
                cur_buffer: 0,
                callback,
                frame_desc,
            });
            let data = Box::into_raw(data); // TODO: destroy

            extern "C" fn write_cb(queue: sles::SLAndroidSimpleBufferQueueItf, user: *mut c_void) {
                unsafe {
                    let data = &mut *(user as *mut CallbackData);
                    data.cur_buffer = (data.cur_buffer + 1) % data.buffers.len();
                    let buffer = &mut data.buffers[data.cur_buffer];

                    let stream = api::Stream {
                        properties: api::StreamProperties {
                            channels: data.frame_desc.channels,
                            sample_rate: data.frame_desc.sample_rate,
                            buffer_size: BUFFER_NUM_FRAMES,
                        },
                        buffers: api::StreamBuffers {
                            output: buffer.as_mut_ptr() as _,
                            input: ptr::null(),
                            frames: buffer.len() / 2, // TODO: data.frame_desc_channels_count(), but always 2 now
                        },
                    };

                    (data.callback)(stream); // TODO: sizeof u32
                    ((**queue).Enqueue).unwrap()(
                        queue,
                        buffer.as_mut_ptr() as _,
                        (buffer.len() * 4) as _,
                    );
                }
            }

            let result = (**queue).RegisterCallback.unwrap()(queue, Some(write_cb), data as _);
            info!("sles::Queue::RegisterCallback: {:?}", result);

            // Enqueue one frame to get the ball rolling
            write_cb(queue, data as _);

            Ok(Device {
                _engine: self.engine,
                state,
                _queue: queue,
            })
        }
    }

    pub struct Device {
        state: sles::SLPlayItf,
        _engine: sles::SLEngineItf,
        _queue: sles::SLAndroidSimpleBufferQueueItf,
    }

    impl Device {
        pub unsafe fn start(&self) {
            let result =
                ((**self.state).SetPlayState).unwrap()(self.state, sles::SL_PLAYSTATE_PLAYING as _);

            info!("Device::start {:?}", result);
        }

        #[allow(dead_code)]
        pub unsafe fn pause(&self) {
            let result =
                ((**self.state).SetPlayState).unwrap()(self.state, sles::SL_PLAYSTATE_PAUSED as _);

            info!("Device::pause {:?}", result);
        }

        #[allow(dead_code)]
        pub unsafe fn stop(&self) {
            let result =
                ((**self.state).SetPlayState).unwrap()(self.state, sles::SL_PLAYSTATE_STOPPED as _);

            info!("Device::stop {:?}", result);
        }
    }
}
enum AudioMessage {
    AddSound(usize, Vec<[f32; 2]>),
    PlaySound(usize, bool, f32),
    SetVolume(usize, f32),
    StopSound(usize),
}

enum ControlMessage {
    Pause,
    Resume,
}

unsafe fn audio_thread(rx: mpsc::Receiver<AudioMessage>, rx1: mpsc::Receiver<ControlMessage>) {
    use opensles::api::*;

    use std::collections::HashMap;

    let instance = opensles::Instance::new();

    let mut sounds = HashMap::new();

    struct SoundState {
        id: usize,
        sample: usize,
        looped: bool,
        volume: f32,
        dead: bool,
    }
    let mut mixer_state: Vec<SoundState> = vec![];
    let device = instance
        .create_device(
            DeviceDesc {
                sample_desc: SampleDesc {
                    format: Format::F32,
                    sample_rate: 44100,
                },
            },
            Channels {
                input: 0,
                output: channel_mask::FRONT_LEFT | channel_mask::FRONT_RIGHT,
            },
            Box::new(move |stream| {
                if let Ok(message) = rx.try_recv() {
                    match message {
                        AudioMessage::AddSound(id, data) => {
                            sounds.insert(id, data);
                        }
                        AudioMessage::PlaySound(id, looped, volume) => {
                            // this is not really correct, but mirrors how it works on wasm/pc
                            if let Some(old) = mixer_state.iter().position(|s| s.id == id) {
                                mixer_state.swap_remove(old);
                            }
                            mixer_state.push(SoundState {
                                id,
                                sample: 0,
                                looped,
                                volume,
                                dead: false,
                            });
                        }
                        AudioMessage::SetVolume(id, volume) => {
                            if let Some(old) = mixer_state.iter_mut().find(|s| s.id == id) {
                                old.volume = volume;
                            }
                        }
                        AudioMessage::StopSound(id) => {
                            if let Some(old) = mixer_state.iter().position(|s| s.id == id) {
                                mixer_state.swap_remove(old);
                            }
                        }
                    }
                }

                let properties = stream.properties;
                let num_channels = properties.num_channels();

                let buffer = std::slice::from_raw_parts_mut(
                    stream.buffers.output as *mut f32,
                    stream.buffers.frames as usize * num_channels,
                );

                for dt in 0..stream.buffers.frames as usize {
                    let mut value = [0.0, 0.0];

                    for sound in &mut mixer_state {
                        let sound_data = &sounds[&sound.id];

                        value[0] += sound_data[sound.sample][0] * sound.volume;
                        value[1] += sound_data[sound.sample][1] * sound.volume;
                        sound.sample = sound.sample + 1;

                        if sound.looped {
                            sound.sample = sound.sample % sound_data.len();
                        } else if sound.sample >= sound_data.len() {
                            sound.dead = true;
                        }
                    }
                    mixer_state.retain(|s| s.dead == false);

                    buffer[num_channels * dt as usize] = value[0];
                    buffer[num_channels * dt as usize + 1] = value[1];
                }
            }),
        )
        .unwrap();

    device.start();

    loop {
        let message = rx1.recv().unwrap();
        match message {
            ControlMessage::Pause => {
                info!("opensles Pause active audio device");
                device.pause();
            }
            ControlMessage::Resume => {
                info!("opensles Start active audio device");
                device.start();
            }
        }
    }
}

pub struct AudioContext {
    tx: mpsc::Sender<AudioMessage>,
    tx1: mpsc::Sender<ControlMessage>,
    id: usize,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        let (tx, rx) = mpsc::channel();
        let (tx1, rx1) = mpsc::channel();

        std::thread::spawn(move || unsafe { audio_thread(rx, rx1) });

        AudioContext { tx, tx1, id: 1 }
    }

    pub fn pause(&mut self) {
        self.tx1.send(ControlMessage::Pause).unwrap()
    }

    pub fn resume(&mut self) {
        self.tx1.send(ControlMessage::Resume).unwrap()
    }
}

pub struct Sound {
    id: usize,
}

impl Sound {
    pub fn load(ctx: &mut AudioContext, data: &[u8]) -> Sound {
        let id = ctx.id;

        let mut audio_stream = {
            let file = std::io::Cursor::new(data);
            audrey::Reader::new(file).unwrap()
        };
        let samples = audio_stream
            .frames::<[f32; 2]>()
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        ctx.tx.send(AudioMessage::AddSound(id, samples)).unwrap();
        ctx.id += 1;

        Sound { id }
    }

    pub fn play(&mut self, ctx: &mut AudioContext, params: PlaySoundParams) {
        ctx.tx
            .send(AudioMessage::PlaySound(
                self.id,
                params.looped,
                params.volume,
            ))
            .unwrap();
    }

    pub fn stop(&mut self, ctx: &mut AudioContext) {
        ctx.tx.send(AudioMessage::StopSound(self.id)).unwrap();
    }

    pub fn set_volume(&mut self, ctx: &mut AudioContext, volume: f32) {
        ctx.tx
            .send(AudioMessage::SetVolume(self.id, volume))
            .unwrap();
    }
}
