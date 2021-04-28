const AudioContext = window.AudioContext || window.webkitAudioContext;
let audio_context;
let sounds = {};
let audio_next_handle = 1;

function audio_init() {
    if (audio_context == null) {
        audio_context = new AudioContext();
        audio_listener = audio_context.listener;

        // play empty sound to unlock context on iOS
        let source = audio_context.createBufferSource();
        source.buffer = audio_context.createBuffer(1, 1, 22050);
        source.connect(audio_context.destination);
        source.start(0);
    }
}

function audio_add_buffer(content, content_len) {
    let content_array = wasm_memory.buffer.slice(content, content + content_len);

    let sound_key = audio_next_handle;
    audio_next_handle += 1;

    sounds[sound_key] = {};

    audio_context.decodeAudioData(content_array, function(buffer) {
        sounds[sound_key].buffer = buffer;
    }, function(e) {
        // fail
        console.error("Failed to decode audio buffer", e);
    });
    return sound_key;
}

function audio_source_is_loaded(sound_key) {
    return (sound_key in sounds) && sounds[sound_key].buffer != undefined;
}

function audio_play_buffer(sound_key, volume_l, volume_r, speed, repeat) {
    audio_source_stop(sound_key);

    var sound = sounds[sound_key];
    let source = audio_context.createBufferSource();
    source.loop = repeat;

    let gain_node_l = null;
    let gain_node_r = null;
    let merger = null;

    if (volume_l != 1.0 || volume_r != 1.0) {
        gain_node_l = audio_context.createGain();
        source.connect(gain_node_l);
        gain_node_r = audio_context.createGain();
        source.connect(gain_node_r);

        let merger = audio_context.createChannelMerger(2);
        gain_node_l.connect(merger, 0, 0);
        gain_node_r.connect(merger, 0, 1);
        merger.connect(audio_context.destination);

        gain_node_l.gain.value = volume_l;
        gain_node_r.gain.value = volume_r;

        sound.merger = merger;
    } else {
        source.connect(audio_context.destination);
    }
    source.playbackRate.value = speed;

    sound.source = source;
    sound.gains = [gain_node_l, gain_node_r];

    source.onended = function() {
        source.disconnect();
        if (gain_node_l) {
            gain_node_l.disconnect();
        }
        if (gain_node_r) {
            gain_node_r.disconnect();
        }
        if (merger) {
            merger.disconnect();
        }
    }

    source.buffer = sound.buffer;
    try {
	source.start(0);
    } catch (e) {
	console.error("Error starting sound", e);
    }
}

function audio_source_set_volume(sound_key, volume_l, volume_r) {
    if (!(sound_key in sounds)) {
        return;
    }

    let gain_nodes = sounds[sound_key].gains;
    let ramp_end_time = audio_context.currentTime + 1.0 / 120.0;

    gain_nodes[0].gain.linearRampToValueAtTime(volume_l, ramp_end_time);
    gain_nodes[1].gain.linearRampToValueAtTime(volume_r, ramp_end_time);
}

function audio_source_stop(sound_key) {
    if (!(sound_key in sounds)) {
        console.log("stopping already remvoed sound")
        return;
    }
    
    // sound was not started or already finished by itself, this is fine
    if (sounds[sound_key].source == undefined) {
        return;
    }

    try {
        sounds[sound_key].source.stop();
        sounds[sound_key].source.disconnect();
        for (node of sounds[sound_key].gains) {
            // null is a valid state when no volume manipulation applied to the sound
            if (node != null) {
                node.disconnect();
            }
        }

        if (sounds[sound_key].merger) {
            sounds[sound_key].merger.disconnect();
        }

    }
    catch (e) {
        console.error("Error stopping sound", e);
    }

    delete sounds[sound_key].source;
    delete sounds[sound_key].gains;
}

function register_plugin(importObject) {
    importObject.env.audio_init = audio_init;
    importObject.env.audio_add_buffer = audio_add_buffer;
    importObject.env.audio_play_buffer = audio_play_buffer;
    importObject.env.audio_source_is_loaded = audio_source_is_loaded;
    importObject.env.audio_source_set_volume = audio_source_set_volume;
    importObject.env.audio_source_stop = audio_source_stop;
}

miniquad_add_plugin({ register_plugin, version: "0.1.0", name: "macroquad_audio" });
