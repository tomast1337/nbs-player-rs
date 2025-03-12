use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Panning, PlaybackRate,
    Tween, Value,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use std::{collections::HashMap, io::Cursor};

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, StaticSoundData>,
}

impl AudioEngine {
    pub fn new(extra_sounds: Option<Vec<String>>) -> Self {
        println!("{:?}", extra_sounds);

        let mut sounds = HashMap::new();
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        let bass = Cursor::new(include_bytes!("../assets/bass.ogg") as &[u8]);
        let bd = Cursor::new(include_bytes!("../assets/bd.ogg") as &[u8]);
        let harp = Cursor::new(include_bytes!("../assets/harp.ogg") as &[u8]);
        let snare = Cursor::new(include_bytes!("../assets/snare.ogg") as &[u8]);
        let hat = Cursor::new(include_bytes!("../assets/hat.ogg") as &[u8]);
        let guitar = Cursor::new(include_bytes!("../assets/guitar.ogg") as &[u8]);
        let flute = Cursor::new(include_bytes!("../assets/flute.ogg") as &[u8]);
        let bell = Cursor::new(include_bytes!("../assets/bell.ogg") as &[u8]);
        let icechime = Cursor::new(include_bytes!("../assets/icechime.ogg") as &[u8]);
        let xylobone = Cursor::new(include_bytes!("../assets/xylobone.ogg") as &[u8]);
        let iron_xylophone = Cursor::new(include_bytes!("../assets/iron_xylophone.ogg") as &[u8]);
        let cow_bell = Cursor::new(include_bytes!("../assets/cow_bell.ogg") as &[u8]);
        let didgeridoo = Cursor::new(include_bytes!("../assets/didgeridoo.ogg") as &[u8]);
        let bit = Cursor::new(include_bytes!("../assets/bit.ogg") as &[u8]);
        let banjo = Cursor::new(include_bytes!("../assets/banjo.ogg") as &[u8]);
        let pling = Cursor::new(include_bytes!("../assets/pling.ogg") as &[u8]);

        sounds.insert(0, StaticSoundData::from_cursor(bass).unwrap());
        sounds.insert(1, StaticSoundData::from_cursor(bd).unwrap());
        sounds.insert(2, StaticSoundData::from_cursor(harp).unwrap());
        sounds.insert(3, StaticSoundData::from_cursor(snare).unwrap());
        sounds.insert(4, StaticSoundData::from_cursor(hat).unwrap());
        sounds.insert(5, StaticSoundData::from_cursor(guitar).unwrap());
        sounds.insert(6, StaticSoundData::from_cursor(flute).unwrap());
        sounds.insert(7, StaticSoundData::from_cursor(bell).unwrap());
        sounds.insert(8, StaticSoundData::from_cursor(icechime).unwrap());
        sounds.insert(9, StaticSoundData::from_cursor(xylobone).unwrap());
        sounds.insert(10, StaticSoundData::from_cursor(iron_xylophone).unwrap());
        sounds.insert(11, StaticSoundData::from_cursor(cow_bell).unwrap());
        sounds.insert(12, StaticSoundData::from_cursor(didgeridoo).unwrap());
        sounds.insert(13, StaticSoundData::from_cursor(bit).unwrap());
        sounds.insert(14, StaticSoundData::from_cursor(banjo).unwrap());
        sounds.insert(15, StaticSoundData::from_cursor(pling).unwrap());

        log::info!("Loaded sounds");

        Self { manager, sounds }
    }

    pub fn play_sound(&mut self, sound_id: u32, key: u8, velocity: u8, panning: i8, pitch: i16) {
        // Retrieve the sound data
        let sound = match self.sounds.get(&sound_id) {
            Some(data) => data.clone(),
            None => {
                log::error!("Sound ID {} not found", sound_id);
                return;
            }
        };

        // Calculate the frequency from the MIDI key
        let frequency_ratio = 2.0f64.powf((key as f64 - 69.0) / 12.0);

        // Adjust the playback rate with pitch (cents)
        let pitch_ratio = 2.0f64.powf(pitch as f64 / 1200.0);
        let playback_rate = PlaybackRate(frequency_ratio * pitch_ratio);

        // Map velocity (0–127) to decibels
        let volume = Decibels::from(velocity as f32 / 127.0);

        // Map panning (-100 to 100) to a normalized range (-1.0 to 1.0)
        let pan = Panning(panning as f32 / 100.0);

        // Configure sound settings
        let settings = StaticSoundSettings::new()
            .volume(volume)
            .panning(pan)
            .playback_rate(playback_rate);

        // Play the sound with the specified settings
        if let Err(e) = self.manager.play(sound.clone().with_settings(settings)) {
            log::error!("Failed to play sound: {}", e);
        }
    }
}
