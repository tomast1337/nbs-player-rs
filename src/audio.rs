use crate::note::NoteBlock;
use raylib::{ffi::PlaySound, prelude::*};
use std::collections::{HashMap, VecDeque};

const DEFAULT_AUDIO_CLIPS: [&[u8]; 16] = [
    include_bytes!("../assets/sounds/harp.ogg") as &[u8],
    include_bytes!("../assets/sounds/bass.ogg") as &[u8],
    include_bytes!("../assets/sounds/bd.ogg") as &[u8],
    include_bytes!("../assets/sounds/snare.ogg") as &[u8],
    include_bytes!("../assets/sounds/hat.ogg") as &[u8],
    include_bytes!("../assets/sounds/guitar.ogg") as &[u8],
    include_bytes!("../assets/sounds/flute.ogg") as &[u8],
    include_bytes!("../assets/sounds/bell.ogg") as &[u8],
    include_bytes!("../assets/sounds/icechime.ogg") as &[u8],
    include_bytes!("../assets/sounds/xylobone.ogg") as &[u8],
    include_bytes!("../assets/sounds/iron_xylophone.ogg") as &[u8],
    include_bytes!("../assets/sounds/cow_bell.ogg") as &[u8],
    include_bytes!("../assets/sounds/didgeridoo.ogg") as &[u8],
    include_bytes!("../assets/sounds/bit.ogg") as &[u8],
    include_bytes!("../assets/sounds/banjo.ogg") as &[u8],
    include_bytes!("../assets/sounds/pling.ogg") as &[u8],
];

pub struct AudioClip<'a> {
    pub wave: Wave<'a>,
    pub pitch: f64,
}

pub struct AudioEngine<'a> {
    pub sounds: HashMap<u32, AudioClip<'a>>,
    sound_pool: VecDeque<Sound<'a>>,
    pool_size: usize,
    raylib_audio: &'a RaylibAudio,
}

impl<'a> AudioEngine<'a> {
    pub fn new(raylib_audio: &'a RaylibAudio, extra_sounds: Option<Vec<(&[u8], f64)>>) -> Self {
        let mut sound_files = DEFAULT_AUDIO_CLIPS
            .iter()
            .copied() // Converts &&[u8] to &[u8]
            .map(|clip| (clip, 45.))
            .collect::<Vec<_>>();

        if let Some(extra_sounds) = extra_sounds {
            log::info!("Loaded {} extra sounds", extra_sounds.len());
            sound_files.extend(extra_sounds);
        }

        let pool_size = if cfg!(target_arch = "wasm32") {
            32 // Smaller pool for WASM
        } else {
            128 // Larger pool for other platforms
        };
        log::debug!("Audio pool size: {}", pool_size);

        let mut audio_engine = Self {
            sounds: HashMap::new(),
            sound_pool: VecDeque::with_capacity(pool_size),
            pool_size,
            raylib_audio,
        };

        for (i, sound) in sound_files.iter().enumerate() {
            let wave = raylib_audio
                .new_wave_from_memory(".ogg", sound.0)
                .expect("Failed to load sound");

            audio_engine.sounds.insert(
                i as u32,
                AudioClip {
                    wave,
                    pitch: sound.1,
                },
            );
        }

        log::info!("Loaded {} sounds", audio_engine.sounds.len());

        audio_engine
    }

    /// Fast approximation for 2^x
    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        for note in notes {
            let sound_id = note.instrument;
            let frequency_ratio = note.frequency_ratio;
            let volume = note.volume;
            let pan = note.pan;

            let sound_data = match self.sounds.get(&sound_id) {
                Some(data) => data,
                None => continue,
            };

            // Get a sound from the pool or create a new one
            let sound = if self.sound_pool.len() >= self.pool_size {
                // Pool is full, reuse the oldest sound
                let old_sound = match self.sound_pool.pop_front() {
                    None => {
                        log::warn!("Sound pool is empty, cannot reuse sound");
                        continue;
                    }
                    Some(sound) => sound,
                };

                // Stop the sound if it's playing
                old_sound.stop();
                // Free the old sound

                // Create new sound from the wave data
                self.raylib_audio
                    .new_sound_from_wave(&sound_data.wave)
                    .expect("Failed to create sound")
            } else {
                // Pool has space, create new sound
                self.raylib_audio
                    .new_sound_from_wave(&sound_data.wave)
                    .expect("Failed to create sound")
            };

            // Configure the sound
            sound.set_pan(pan);
            sound.set_volume(volume);
            sound.set_pitch(frequency_ratio);

            // Play the sound
            unsafe {
                PlaySound(sound.clone());
            }

            // Add to pool
            self.sound_pool.push_back(sound);
        }
    }
}
