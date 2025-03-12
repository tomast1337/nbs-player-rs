use kira::{
    self, AudioManager, AudioManagerSettings, DefaultBackend, Frame,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use lewton::inside_ogg::OggStreamReader;
use rubato::{FftFixedInOut, VecResampler};
use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::note::NoteBlock;
#[derive(Debug, Clone)]
struct Mixer(Vec<SoundData>);
impl Mixer {
    fn mix_samples(&self) -> Arc<[Frame]> {
        // Find the longest sample to determine final length
        let max_length = (&self.0).iter().map(|s| s.0.len()).max().unwrap_or(0);
        let mut mixed = vec![0.0; max_length];
        // Mix all samples together
        for samples in &self.0 {
            for (i, &sample) in samples.0.iter().enumerate() {
                mixed[i] += sample;
            }
        }
        // Normalize to prevent clipping
        let max_amplitude = mixed.iter().fold(0.0, |max, &s| s.abs().max(max));
        if max_amplitude > 1.0 {
            mixed.iter_mut().for_each(|s| *s /= max_amplitude);
        }

        // Convert to frames
        let frames = mixed.iter().map(|&s| Frame::new(s, s)).collect::<Vec<_>>();
        return frames.into();
    }
}
#[derive(Debug, Clone)]
struct SoundData(Vec<f32>, u32); // Samples, sample rate

impl SoundData {
    pub fn new(data: &[u8]) -> Result<SoundData, lewton::VorbisError> {
        let cursor = Cursor::new(data);
        let mut stream = OggStreamReader::new(cursor).expect("Failed to read Ogg stream");
        let mut samples = Vec::new();
        while let Ok(Some(pcm)) = stream.read_dec_packet_itl() {
            samples.extend(
                pcm.into_iter()
                    .map(|s| s as f32 / f32::from(i16::MAX)) // Normalize to [-1.0, 1.0]
                    .collect::<Vec<f32>>(),
            );
        }
        return Ok(SoundData(samples, stream.ident_hdr.audio_sample_rate));
    }

    pub fn apply_volume_and_panning(&mut self, volume: f32, pan: f32) -> &mut Self {
        assert!(
            self.0.len() % 2 == 0,
            "Samples must be interleaved stereo pairs"
        );
        assert!(
            volume >= 0.0 && volume <= 1.0,
            "Volume must be in range [0.0, 1.0]"
        );
        for i in (0..self.0.len()).step_by(2) {
            let left = volume * (1.0 - pan);
            let right = volume * pan;

            self.0[i] *= left; // Left channel
            self.0[i + 1] *= right; // Right channel
        }
        return self;
    }

    pub fn change_pitch(&mut self, pitch_factor: f32) -> &mut Self {
        let target_sample_rate = (self.1 as f32 * pitch_factor) as usize;

        // Ensure the input data is valid stereo
        assert!(
            self.0.len() % 2 == 0,
            "Samples must be interleaved stereo pairs"
        );

        // Prepare input frames as stereo pairs
        let input_frames: Vec<Vec<f32>> = self
            .0
            .chunks_exact(2)
            .map(|chunk| vec![chunk[0], chunk[1]])
            .collect();

        // Pad input frames to align with the resampler's chunk size
        let chunk_size = 1024;
        let padding_needed = (chunk_size - (input_frames.len() % chunk_size)) % chunk_size;
        let mut padded_input_frames = input_frames.clone();
        padded_input_frames.extend(vec![vec![0.0, 0.0]; padding_needed]);

        // Initialize the resampler
        let mut resampler = FftFixedInOut::<f32>::new(
            self.1 as usize,    // Input sample rate
            target_sample_rate, // Output sample rate
            chunk_size,         // Chunk size
            2,                  // Stereo channels
        )
        .expect("Failed to create resampler");

        // Resample the audio
        let output_frames = resampler
            .process(&padded_input_frames, None)
            .expect("Failed to resample");

        // Flatten the output frames back into interleaved stereo samples
        self.0 = output_frames
            .iter()
            .flat_map(|frame| frame.iter().cloned())
            .collect();

        // Update the sample rate
        self.1 = target_sample_rate as u32;

        return self;
    }

    pub fn encode(&mut self) -> Arc<[Frame]> {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(
            &mut cursor,
            hound::WavSpec {
                channels: 2,
                sample_rate: self.1 as u32,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .expect("Failed to create WAV writer");

        for &sample in self.0.iter() {
            writer.write_sample(sample).expect("Failed to write sample");
        }

        writer.finalize().expect("Failed to finalize WAV file");

        let data = cursor.into_inner();
        let reader = hound::WavReader::new(Cursor::new(data)).expect("Failed to read WAV file");
        let frames = reader
            .into_samples::<f32>()
            .map(|s| {
                let sample = s.expect("Failed to read sample");
                Frame::new(sample, sample)
            })
            .collect::<Vec<_>>();

        return frames.into();
    }
}

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sounds: HashMap<u32, SoundData>,
}

impl AudioEngine {
    pub fn new(extra_sounds: Option<Vec<&[u8]>>) -> Self {
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
            sounds.insert(
                i as u32,
                SoundData::new(sound).expect("Failed to load sound data"),
            );
        }

        log::info!("Loaded {} sounds", sounds.len());

        Self { manager, sounds }
    }

    pub fn play_sound(&mut self, note: &NoteBlock) {
        let sound_id = note.instrument as u32;
        let key = note.key;
        let velocity = note.velocity;
        let panning = note.panning;
        let pitch = note.pitch;

        // Calculate the frequency from the MIDI key
        let frequency_ratio = 2.0f32.powf((key as f32 - 69.0) / 12.0);

        // Adjust the playback rate with pitch (cents)
        let pitch_ratio = 2.0f32.powf(pitch as f32 / 1200.0);
        let playback_rate = frequency_ratio * pitch_ratio;

        // Map velocity (0–127) to decibels
        let volume = velocity as f32 / 127.0;

        // Map panning (-100 to 100) to a normalized range (-1.0 to 1.0)
        let pan = panning as f32 / 100.;

        // Retrieve the sound data
        let mut sound = match self.sounds.get(&sound_id) {
            Some(data) => data.clone(),
            None => {
                log::error!("Sound ID {} not found", sound_id);
                return;
            }
        };

        let encoded = sound
            .apply_volume_and_panning(volume, pan)
            //.change_pitch(playback_rate)
            .encode();

        let sound = StaticSoundData {
            sample_rate: sound.1,
            frames: encoded,
            settings: StaticSoundSettings::new(), // Default settings for sound playback
            slice: None,
        };

        // Play the sound
        if let Err(e) = self.manager.play(sound) {
            log::error!("Failed to play sound: {}", e);
        }
    }

    pub fn play_tick(&mut self, notes: &[NoteBlock]) {
        let mut samples = Vec::new();

        for note in notes {
            let sound_id = note.instrument as u32;
            let key = note.key;
            let velocity = note.velocity;
            let panning = note.panning;
            let pitch = note.pitch;

            // Calculate the frequency from the MIDI key
            let frequency_ratio = 2.0f32.powf((key as f32 - 69.0) / 12.0);

            // Adjust the playback rate with pitch (cents)
            let pitch_ratio = 2.0f32.powf(pitch as f32 / 1200.0);
            let playback_rate = frequency_ratio * pitch_ratio;

            // Map velocity (0–127) to decibels
            let volume = velocity as f32 / 127.0;

            // Map panning (-100 to 100) to a normalized range (-1.0 to 1.0)
            let pan = panning as f32 / 100.0;

            // Retrieve the sound data
            let sound = match self.sounds.get(&sound_id) {
                Some(data) => data.clone(),
                None => {
                    log::error!("Sound ID {} not found", sound_id);
                    return;
                }
            };

            let mut processed_sound = sound.clone();
            processed_sound.apply_volume_and_panning(volume, pan);
            //.change_pitch(playback_rate);

            samples.push(processed_sound);
        }

        let mixer = Mixer(samples);

        let final_sample = mixer.mix_samples();

        let sound = StaticSoundData {
            sample_rate: 44100,
            frames: final_sample.into(),
            settings: StaticSoundSettings::new(), // Default settings for sound playback
            slice: None,
        };

        // Play the sound
        if let Err(e) = self.manager.play(sound) {
            log::error!("Failed to play sound: {}", e);
        }
    }
}
