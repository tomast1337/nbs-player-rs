use crate::note::NoteBlock;
use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Mix, Panning,
    PlaybackRate,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    track::{TrackBuilder, TrackHandle},
};
use std::{collections::HashMap, io::Cursor, vec};

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, StaticSoundData>,
    global_volume: f32,
    main_track: TrackHandle,
}

impl AudioEngine {
    fn load_sound_data(data: Vec<u8>) -> StaticSoundData {
        let cursor = Cursor::new(data);
        let sound_data = StaticSoundData::from_cursor(cursor).expect("Failed to load sound data");
        sound_data
    }

    pub fn new(extra_sounds: Option<Vec<&[u8]>>, global_volume: f32) -> Self {
        let mut manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        let mut sound_files = vec![
            include_bytes!("../assets/harp.ogg") as &[u8], //0 = Piano (Air)
            include_bytes!("../assets/bass.ogg") as &[u8], //1 = Double Bass (Wood)
            include_bytes!("../assets/bd.ogg") as &[u8],   //2 = Bass Drum (Stone)
            include_bytes!("../assets/snare.ogg") as &[u8], //3 = Snare Drum (Sand)
            include_bytes!("../assets/hat.ogg") as &[u8],  //4 = Click (Glass)
            include_bytes!("../assets/guitar.ogg") as &[u8], //5 = Guitar (Wool)
            include_bytes!("../assets/flute.ogg") as &[u8], //6 = Flute (Clay)
            include_bytes!("../assets/bell.ogg") as &[u8], //7 = Bell (Block of Gold)
            include_bytes!("../assets/icechime.ogg") as &[u8], //8 = Chime (Packed Ice)
            include_bytes!("../assets/xylobone.ogg") as &[u8], //9 = Xylophone (Bone Block)
            include_bytes!("../assets/iron_xylophone.ogg") as &[u8], //10 = Iron Xylophone (Iron Block)
            include_bytes!("../assets/cow_bell.ogg") as &[u8],       //11 = Cow Bell (Soul Sand)
            include_bytes!("../assets/didgeridoo.ogg") as &[u8],     //12 = Didgeridoo (Pumpkin)
            include_bytes!("../assets/bit.ogg") as &[u8],            //13 = Bit (Block of Emerald)
            include_bytes!("../assets/banjo.ogg") as &[u8],          //14 = Banjo (Hay)
            include_bytes!("../assets/pling.ogg") as &[u8],          //15 = Pling (Glowstone)
        ];

        if let Some(extra_sounds) = extra_sounds {
            log::info!("Loaded {} extra sounds", extra_sounds.len());
            sound_files.extend(extra_sounds);
        }

        let mut sounds = HashMap::new();

        for (i, sound) in sound_files.iter().enumerate() {
            sounds.insert(i as u32, AudioEngine::load_sound_data(sound.to_vec()));
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
            sample_rate: sound_data.sample_rate,
            frames: sound_data.frames.clone(),
            settings: StaticSoundSettings::default(),
            slice: None,
        };

        let frequency_ratio = 2.0f64.powf((key + (pitch / 100.0) - 45.) / 12.0);
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
