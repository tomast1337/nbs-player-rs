use crate::note::NoteBlock;
use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Panning, PlaybackRate,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    track::{TrackBuilder, TrackHandle},
};

use std::{collections::HashMap, io::Cursor, vec};

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, (StaticSoundData, f64)>,
    global_volume: f32,
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
            manager,
            sounds,
            global_volume,
        }
    }

    fn get_sound_data(&mut self, note: &NoteBlock) -> Option<StaticSoundData> {
        let sound_id = note.instrument as u32;
        let key = note.key as f64;
        let velocity = note.velocity as f32;
        let panning = note.panning as f32;
        let pitch = note.pitch as f64;
        let sound_data = match self.sounds.get(&sound_id) {
            Some(data) => data,
            None => {
                log::error!("Sound ID {} not found", sound_id);
                return None;
            }
        };
        let sound = StaticSoundData {
            sample_rate: sound_data.0.sample_rate,
            frames: sound_data.0.frames.clone(),
            settings: StaticSoundSettings::default(),
            slice: None,
        };

        let tone = sound_data.1;

        let frequency_ratio = 2.0f64.powf((key + (pitch / 100.0) - tone) / 12.0);
        let playback_rate = PlaybackRate(frequency_ratio);

        let epoch = 1e-6;

        let volume: Decibels = Decibels::from(
            10.0 * ((velocity * self.global_volume + epoch) / (100.0 + epoch)).log10(),
        );

        let pan = Panning((panning / 100.0) - 1.);

        let settings = StaticSoundSettings::default()
            .volume(volume)
            .panning(pan)
            .playback_rate(playback_rate);
        // get resampling sound with the specified settings
        let sample = sound.clone().with_settings(settings.clone());

        Some(sample)
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
