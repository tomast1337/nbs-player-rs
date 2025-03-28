use crate::note::NoteBlock;
use raylib::{ffi::PlaySound, prelude::*};
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

pub struct AudioClip<'a> {
    pub wave: Wave<'a>, // Store the Wave instead of Sound
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
        let data = vec![
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
        let mut sound_files = data.iter().map(|data| (*data, 45.)).collect::<Vec<_>>();

        if let Some(extra_sounds) = extra_sounds {
            log::info!("Loaded {} extra sounds", extra_sounds.len());
            sound_files.extend(extra_sounds);
        }

        let pool_size = if cfg!(target_arch = "wasm32") {
            128 // Smaller pool for WASM
        } else {
            256 // Larger pool for other platforms
        };
        log::debug!("Pool size: {}", pool_size);

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
    pub fn fast_pow2(x: f32) -> f32 {
        let x0 = x.floor();
        let x1 = x - x0;

        // Handle overflow and underflow
        if x0 >= 32.0 {
            return f32::INFINITY; // 2^x is too large for f32
        } else if x0 <= -32.0 {
            return 0.0; // 2^x is too small for f32
        }

        // Calculate 2^x1 using a polynomial approximation
        let p = 1.0 + x1 * (0.693147 + x1 * (0.241586 + x1 * 0.052043));

        // Calculate 2^x0 using bit shifting (only for positive x0)
        if x0 >= 0.0 {
            p * (1 << x0 as i32) as f32
        } else {
            p / (1 << (-x0 as i32)) as f32
        }
    }
    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        const INV_12: f32 = 1.0 / 12.0;

        for note in notes {
            let sound_id = note.instrument as u32;
            let key = note.key as f32;
            let velocity = note.velocity as f32;
            let panning = note.panning as f32;
            let pitch = note.pitch as f32;

            let sound_data = match self.sounds.get(&sound_id) {
                Some(data) => data,
                None => continue,
            };

            // Calculate sound properties
            let tone = sound_data.pitch as f32;
            let frequency_ratio = AudioEngine::fast_pow2((key + (pitch / 100.0) - tone) * INV_12);
            let volume = velocity / 100.0;
            let pan = ((panning + 100.0) / 200.0) - 0.5;

            // Get a sound from the pool or create a new one
            let sound = if self.sound_pool.len() >= self.pool_size {
                // Pool is full, reuse the oldest sound
                let old_sound = self.sound_pool.pop_front().unwrap();

                // Stop the sound if it's playing
                old_sound.stop();

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
