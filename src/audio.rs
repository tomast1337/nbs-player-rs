use crate::note::NoteBlock;
use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Panning, PlaybackRate,
    Tween,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    track::{TrackBuilder, TrackHandle},
};

use std::{collections::HashMap, io::Cursor, vec};

pub struct AudioEngine {
    _manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, (StaticSoundData, f64)>,
    main_track: TrackHandle,
}

impl AudioEngine {
    fn load_sound_data(data: Vec<u8>) -> StaticSoundData {
        let cursor = Cursor::new(data);
        let sound_data = StaticSoundData::from_cursor(cursor).expect("Failed to load sound data");
        sound_data
    }

    pub fn new(extra_sounds: Option<Vec<(&[u8], f64)>>, global_volume: f32) -> Self {
        let mut manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

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

        let mut sounds = HashMap::new();

        for (i, sound) in sound_files.iter().enumerate() {
            sounds.insert(i as u32, (Self::load_sound_data(sound.0.to_vec()), sound.1));
        }

        log::info!("Loaded {} sounds", sounds.len());
        let main_track = manager
            .add_sub_track(TrackBuilder::new().volume(global_volume))
            .unwrap();

        Self {
            main_track,
            _manager: manager,
            sounds,
        }
    }

    fn get_sound_data(&mut self, note: &NoteBlock) -> Option<StaticSoundData> {
        // Fast approximation for 2^x
        fn fast_pow2(x: f32) -> f32 {
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

        // Precompute constants
        const INV_12: f32 = 1.0 / 12.0;
        const EPOCH: f32 = 1e-6;

        // Extract note properties
        let sound_id = note.instrument as u32;
        let key = note.key as f32; // Use f32 directly
        let velocity = note.velocity as f32;
        let panning = note.panning as f32;
        let pitch = note.pitch as f32; // Use f32 directly

        // Fetch sound data
        let sound_data = match self.sounds.get(&sound_id) {
            Some(data) => data,
            None => {
                log::error!("Sound ID {} not found", sound_id);
                return None;
            }
        };

        // Clone sound data
        let sound = StaticSoundData {
            sample_rate: sound_data.0.sample_rate,
            frames: sound_data.0.frames.clone(),
            settings: StaticSoundSettings::default(),
            slice: None,
        };

        // Calculate frequency ratio using fast approximation
        let tone = sound_data.1 as f32;
        let frequency_ratio = fast_pow2((key + (pitch / 100.0) - tone) * INV_12);
        let playback_rate = PlaybackRate(frequency_ratio as f64);

        // Calculate volume in decibels
        let volume = Decibels::from(10.0 * ((velocity + EPOCH) / (100.0 + EPOCH)).log10());

        // Calculate panning
        let pan = Panning((panning / 100.0) - 1.0);

        // Apply settings
        let settings = StaticSoundSettings::default()
            .volume(volume)
            .panning(pan)
            .playback_rate(playback_rate);

        // Return resampled sound with the specified settings
        Some(sound.clone().with_settings(settings))
    }

    pub fn set_global_volume(&mut self, volume: f32) {
        // Ensure volume is between 0 to 100
        let volume = if volume < 0.0 {
            0.0
        } else if volume > 1.0 {
            1.0
        } else {
            volume
        };

        self.main_track.set_volume(
            volume,
            Tween {
                ..Default::default()
            },
        );
    }

    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        for note in notes {
            if let Some(sound) = self.get_sound_data(note) {
                if let Err(e) = self.main_track.play(sound) {
                    log::error!("Failed to play sound: {}", e);
                }
            }
        }
    }
}
