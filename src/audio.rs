use crate::note::NoteBlock;
use dasp::sample;
use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Mix, Panning,
    PlaybackRate,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use lewton::samples;
use std::{collections::HashMap, io::Cursor, vec};

#[derive(Debug, Clone)]
struct Mixer;
impl Mixer {
    fn mix_samples(
        samples: Vec<StaticSoundData>,
        sample_rate: u32,
        global_volume: f32,
    ) -> StaticSoundData {
        let max_len = samples.iter().map(|s| s.frames.len()).max().unwrap();
        let mut final_samples = vec![
            Frame {
                left: 0.0,
                right: 0.0,
            };
            max_len
        ];

        for sample in samples.iter() {
            sample.frames.iter().enumerate().for_each(|(i, frame)| {
                final_samples[i].left += frame.left;
                final_samples[i].right += frame.right;
            });
        }

        // apply limiter to prevent clipping
        let max = final_samples
            .iter()
            .map(|frame| frame.left.abs().max(frame.right.abs()))
            .fold(0.0, f32::max);
        let limiter = global_volume / max;

        final_samples.iter_mut().for_each(|frame| {
            frame.left *= limiter;
            frame.right *= limiter;
        });

        StaticSoundData {
            sample_rate,
            frames: final_samples.into(),
            settings: StaticSoundSettings::default(),
            slice: None,
        }
    }
}

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, StaticSoundData>,
    global_volume: f32,
}

impl AudioEngine {
    fn load_sound_data(data: Vec<u8>) -> StaticSoundData {
        let cursor = Cursor::new(data);
        let sound_data = StaticSoundData::from_cursor(cursor).expect("Failed to load sound data");
        sound_data
    }

    pub fn new(extra_sounds: Option<Vec<&[u8]>>, global_volume: f32) -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        let mut sound_files = vec![
            include_bytes!("../assets/bass.ogg") as &[u8],
            include_bytes!("../assets/bd.ogg") as &[u8],
            include_bytes!("../assets/harp.ogg") as &[u8],
            include_bytes!("../assets/snare.ogg") as &[u8],
            include_bytes!("../assets/hat.ogg") as &[u8],
            include_bytes!("../assets/guitar.ogg") as &[u8],
            include_bytes!("../assets/flute.ogg") as &[u8],
            include_bytes!("../assets/bell.ogg") as &[u8],
            include_bytes!("../assets/icechime.ogg") as &[u8],
            include_bytes!("../assets/xylobone.ogg") as &[u8],
            include_bytes!("../assets/iron_xylophone.ogg") as &[u8],
            include_bytes!("../assets/cow_bell.ogg") as &[u8],
            include_bytes!("../assets/didgeridoo.ogg") as &[u8],
            include_bytes!("../assets/bit.ogg") as &[u8],
            include_bytes!("../assets/banjo.ogg") as &[u8],
            include_bytes!("../assets/pling.ogg") as &[u8],
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

        Self {
            manager,
            sounds,
            global_volume,
        }
    }

    pub fn _play_sound(&mut self, note: &NoteBlock) {
        let sample = match self.get_sound_data(note) {
            Some(value) => value,
            None => return,
        };

        let _ = sample.volume(self.global_volume);

        // Play the sound with the specified settings
        if let Err(e) = self.manager.play(sample) {
            log::error!("Failed to play sound: {}", e);
        }
    }

    fn get_sound_data(&mut self, note: &NoteBlock) -> Option<StaticSoundData> {
        let sound_id = note.instrument as u32;
        let key = note.key;
        let velocity = note.velocity;
        let panning = note.panning;
        let pitch = note.pitch;
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
        let frequency_ratio = 2.0f64.powf((key as f64 - 69.0) / 12.0);
        let pitch_ratio = 2.0f64.powf(pitch as f64 / 1200.0);
        let playback_rate = PlaybackRate(frequency_ratio * pitch_ratio);
        let volume = Decibels::from(velocity as f32 / 127.0);
        let pan = Panning(panning as f32 / 100.0);
        let settings = StaticSoundSettings::new()
            .volume(volume)
            .panning(pan)
            .playback_rate(playback_rate);
        // get resampling sound with the specified settings
        let sample = sound.clone().with_settings(settings.clone());

        Some(sample)
    }

    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        if notes.is_empty() {
            return;
        }

        let mut samples = Vec::new();

        for note in notes {
            let sample = match self.get_sound_data(note) {
                Some(value) => value,
                None => continue,
            };

            samples.push(sample);
        }

        // Mix all the sounds together
        let mixed = Mixer::mix_samples(samples, 44100, self.global_volume);

        // Play the sound with the specified settings
        if let Err(e) = self.manager.play(mixed) {
            log::error!("Failed to play sound: {}", e);
        }
    }
}
