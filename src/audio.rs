use dasp::Sample;
use kira::{
    self, AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Panning,
    PlaybackRate, Tween, Value,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use lewton::inside_ogg::OggStreamReader;
use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::note::NoteBlock;

struct SoundData {
    frames: Arc<[Frame]>,
    sample_rate: u32,
}

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, SoundData>,
}

impl AudioEngine {
    pub fn new(extra_sounds: Option<Vec<String>>) -> Self {
        println!("{:?}", extra_sounds);

        let mut sounds = HashMap::new();
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        let sound_files = vec![
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

        for (i, sound) in sound_files.iter().enumerate() {
            let cursor = Cursor::new(sound);
            let mut stream = OggStreamReader::new(cursor).expect("Failed to read Ogg stream");
            let mut samples = Vec::new();
            while let Ok(Some(pcm)) = stream.read_dec_packet_itl() {
                samples.extend(
                    pcm.into_iter()
                        .map(|s| s as f32 / f32::from(i16::MAX)) // Normalize to [-1.0, 1.0]
                        .collect::<Vec<f32>>(),
                );
            }

            // Convert samples to Frame based on channel count
            let channels = stream.ident_hdr.audio_channels as usize;
            let frames_vec: Vec<Frame> = match channels {
                1 => {
                    // Mono: each sample becomes a Frame with same left/right
                    samples.iter().map(|s| Frame::from_mono(*s)).collect()
                }
                2 => {
                    // Stereo: pair samples into left/right channels
                    samples
                        .chunks(2)
                        .map(|chunk| Frame::new(chunk[0], chunk[1]))
                        .collect()
                }
                _ => panic!("Unsupported channel count: {}", channels),
            };
            let frames: Arc<[Frame]> = Arc::from(frames_vec.as_slice());
            let sample_rate = stream.ident_hdr.audio_sample_rate;

            sounds.insert(
                i as u32,
                SoundData {
                    frames,
                    sample_rate,
                },
            );
        }

        log::info!("Loaded sounds");

        Self { manager, sounds }
    }

    pub fn play_sound(&mut self, note: &NoteBlock) {
        let sound_id = note.instrument as u32;
        let key = note.key;
        let velocity = note.velocity;
        let panning = note.panning;
        let pitch = note.pitch;

        // Retrieve the sound data
        let sound_data = match self.sounds.get(&sound_id) {
            Some(data) => data,
            None => {
                log::error!("Sound ID {} not found", sound_id);
                return;
            }
        };

        let sound = StaticSoundData {
            sample_rate: sound_data.sample_rate,
            frames: sound_data.frames.clone(),
            settings: StaticSoundSettings::default(),
            slice: None,
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

    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        if notes.is_empty() {
            return; // Nothing to play
        }

        // Step 1: Determine the maximum length of the mixed output
        let mut max_frames = 0;
        let mut sample_rate = 0;
        for note in notes {
            if let Some(sound_data) = self.sounds.get(&(note.instrument as u32)) {
                let playback_rate = Self::calculate_playback_rate(note.key, note.pitch);
                let adjusted_length = (sound_data.frames.len() as f64 / playback_rate.0) as usize;
                max_frames = max_frames.max(adjusted_length);
                sample_rate = sound_data.sample_rate; // Assume consistent sample rate
            }
        }

        // Step 2: Mix all notes into a single Vec<Frame>
        let mut mixed_frames = vec![Frame::ZERO; max_frames];
        for note in notes {
            let sound_id = note.instrument;
            let sound_data = match self.sounds.get(&(sound_id as u32)) {
                Some(data) => data,
                None => {
                    log::error!("Sound ID {} not found", sound_id);
                    continue;
                }
            };

            let playback_rate = Self::calculate_playback_rate(note.key, note.pitch);
            let volume_f32 = note.velocity as f32 / 127.0; // Linear gain
            let panning = Panning(note.panning as f32 / 100.0);

            // Resample and mix frames
            let frame_count = sound_data.frames.len();
            for i in 0..max_frames {
                let src_idx = (i as f64 * playback_rate.0) as usize;
                if src_idx < frame_count {
                    let frame = sound_data.frames[src_idx]
                        .panned(panning) // Apply panning
                        * volume_f32; // Apply volume (linear multiplication)
                    mixed_frames[i].left += frame.left;
                    mixed_frames[i].right += frame.right;
                }
            }
        }

        // Step 3: Apply a simple limiter (clamp to [-1.0, 1.0])
        for frame in &mut mixed_frames {
            frame.left = frame.left.clamp(-1.0, 1.0);
            frame.right = frame.right.clamp(-1.0, 1.0);
        }

        // Step 4: Create and play the mixed sound
        let sound = StaticSoundData {
            sample_rate,
            frames: Arc::from(mixed_frames.as_slice()),
            settings: StaticSoundSettings::new(), // Default settings for tick playback
            slice: None,
        };

        if let Err(e) = self.manager.play(sound) {
            log::error!("Failed to play mixed tick: {}", e);
        }
    }

    // Helper function to calculate playback rate
    fn calculate_playback_rate(key: u8, pitch: i16) -> PlaybackRate {
        let frequency_ratio = 2.0f64.powf((key as f64 - 69.0) / 12.0);
        let pitch_ratio = 2.0f64.powf(pitch as f64 / 1200.0);
        PlaybackRate(frequency_ratio * pitch_ratio)
    }
}
