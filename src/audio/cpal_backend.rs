use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream};
use nbs_player_core::audio::{AudioBackend, Instrument, SharedMixer};
use nbs_player_core::notes::NoteBlock;
use nbs_player_core::profiler;

/// Software mixer (from the core crate) pulled by a cpal output stream. No per-note
/// allocation happens on the game thread: notes are just voice start events.
pub struct CpalBackend {
    mixer: SharedMixer,
    _stream: Stream,
}

impl CpalBackend {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no default output device")?;
        let config = device
            .default_output_config()
            .map_err(|e| e.to_string())?;
        let sample_rate = config.sample_rate();
        let mixer = SharedMixer::new(sample_rate);
        log::info!("cpal output: {} Hz, {:?}", sample_rate, config.sample_format());

        let stream = match config.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, config.into(), &mixer),
            SampleFormat::F64 => build::<f64>(&device, config.into(), &mixer),
            SampleFormat::I16 => build::<i16>(&device, config.into(), &mixer),
            SampleFormat::I32 => build::<i32>(&device, config.into(), &mixer),
            SampleFormat::U16 => build::<u16>(&device, config.into(), &mixer),
            other => Err(format!("unsupported sample format {other}")),
        }?;
        stream.play().map_err(|e| e.to_string())?;

        Ok(Self {
            mixer,
            _stream: stream,
        })
    }
}

fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mixer: &SharedMixer,
) -> Result<Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let mixer = mixer.clone();
    let mut scratch: Vec<f32> = Vec::new();

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                let frames = data.len() / channels;
                scratch.resize(frames * 2, 0.0);
                mixer.mix_stereo(&mut scratch);
                for (frame, lr) in data.chunks_mut(channels).zip(scratch.chunks_exact(2)) {
                    match channels {
                        1 => frame[0] = T::from_sample((lr[0] + lr[1]) * 0.5),
                        _ => {
                            frame[0] = T::from_sample(lr[0]);
                            frame[1] = T::from_sample(lr[1]);
                            for extra in &mut frame[2..] {
                                *extra = T::EQUILIBRIUM;
                            }
                        }
                    }
                }
            },
            |err| log::error!("cpal stream error: {err}"),
            None,
        )
        .map_err(|e| e.to_string())
}

impl AudioBackend for CpalBackend {
    fn load_instrument(&mut self, instrument: &Instrument) {
        self.mixer.load_instrument(instrument);
    }

    fn play(&mut self, note: &NoteBlock) {
        self.mixer.play(note);
    }

    fn play_tick(&mut self, notes: &[NoteBlock]) {
        let _p = profiler::scope("play_tick");
        self.mixer.play_tick(notes);
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.mixer.set_master_volume(volume);
    }
}
