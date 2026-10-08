use std::collections::{HashMap, VecDeque};

use nbs_player_core::audio::{AudioBackend, Instrument};
use nbs_player_core::notes::NoteBlock;
use nbs_player_core::profiler;
use raylib::{ffi::PlaySound, prelude::*};

/// One raylib `Sound` is created per note and kept in a bounded pool; the oldest is
/// stopped when the pool is full. This is the original (pre-split) behaviour.
pub struct RaylibBackend<'a> {
    waves: HashMap<u32, Wave<'a>>,
    sound_pool: VecDeque<Sound<'a>>,
    pool_size: usize,
    audio: &'a RaylibAudio,
}

impl<'a> RaylibBackend<'a> {
    pub fn new(audio: &'a RaylibAudio) -> Self {
        let pool_size = if cfg!(target_arch = "wasm32") { 32 } else { 128 };
        Self {
            waves: HashMap::new(),
            sound_pool: VecDeque::with_capacity(pool_size),
            pool_size,
            audio,
        }
    }
}

impl<'a> AudioBackend for RaylibBackend<'a> {
    fn load_instrument(&mut self, instrument: &Instrument) {
        match self.audio.new_wave_from_memory(".ogg", &instrument.ogg) {
            Ok(wave) => {
                self.waves.insert(instrument.id, wave);
            }
            Err(e) => log::warn!("instrument {} skipped: {}", instrument.id, e),
        }
    }

    fn play(&mut self, note: &NoteBlock) {
        let Some(wave) = self.waves.get(&note.instrument) else {
            return;
        };

        if self.sound_pool.len() >= self.pool_size {
            if let Some(old) = self.sound_pool.pop_front() {
                old.stop();
            }
        }

        let sound = {
            let _p = profiler::scope("new_sound_from_wave");
            match self.audio.new_sound_from_wave(wave) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("failed to create sound: {e}");
                    return;
                }
            }
        };

        // raylib pans 0..1 around 0.5; the historical mapping fed it pan/2.
        sound.set_pan(note.pan * 0.5);
        sound.set_volume(note.volume);
        sound.set_pitch(note.frequency_ratio);
        unsafe {
            PlaySound(sound.clone());
        }
        self.sound_pool.push_back(sound);
    }

    fn play_tick(&mut self, notes: &[NoteBlock]) {
        let _p = profiler::scope("play_tick");
        for note in notes {
            self.play(note);
        }
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.audio.set_master_volume(volume);
    }
}
