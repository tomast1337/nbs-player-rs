use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{AudioBackend, Instrument, Pcm};
use crate::notes::NoteBlock;

const MAX_VOICES: usize = if cfg!(target_arch = "wasm32") { 64 } else { 128 };

struct Voice {
    pcm: Arc<Pcm>,
    /// Read position in source samples.
    pos: f64,
    /// Source samples consumed per output frame.
    step: f64,
    left: f32,
    right: f32,
    /// Start order; the lowest is stolen first when all voices are busy.
    serial: u64,
}

/// Polyphonic software mixer. Produces interleaved stereo `f32` and does no I/O, so any
/// output (cpal, a Web Audio worklet, a WAV writer) can drive it.
pub struct Mixer {
    instruments: HashMap<u32, Arc<Pcm>>,
    voices: Vec<Voice>,
    max_voices: usize,
    out_rate: f64,
    master: f32,
    serial: u64,
}

impl Mixer {
    pub fn new(out_sample_rate: u32) -> Self {
        Self {
            instruments: HashMap::new(),
            voices: Vec::with_capacity(MAX_VOICES),
            max_voices: MAX_VOICES,
            out_rate: out_sample_rate.max(1) as f64,
            master: 1.0,
            serial: 0,
        }
    }

    pub fn set_instrument(&mut self, id: u32, pcm: Arc<Pcm>) {
        self.instruments.insert(id, pcm);
    }

    pub fn active_voices(&self) -> usize {
        self.voices.len()
    }

    pub fn set_master_volume(&mut self, v: f32) {
        self.master = v.clamp(0.0, 1.0);
    }

    pub fn play(&mut self, id: u32, frequency_ratio: f32, volume: f32, pan: f32) {
        let Some(pcm) = self.instruments.get(&id) else {
            return;
        };
        if pcm.samples.len() < 2 || !frequency_ratio.is_finite() || frequency_ratio <= 0.0 {
            return;
        }
        let pan = pan.clamp(-1.0, 1.0);
        let volume = volume.max(0.0);
        self.serial += 1;
        let voice = Voice {
            step: frequency_ratio as f64 * pcm.sample_rate as f64 / self.out_rate,
            pcm: Arc::clone(pcm),
            pos: 0.0,
            left: volume * (1.0 - pan).min(1.0),
            right: volume * (1.0 + pan).min(1.0),
            serial: self.serial,
        };
        if self.voices.len() < self.max_voices {
            self.voices.push(voice);
        } else if let Some(oldest) = self.voices.iter_mut().min_by_key(|v| v.serial) {
            *oldest = voice;
        }
    }

    /// Fill `out` (interleaved stereo, `len` must be even) with the next frames.
    pub fn mix_stereo(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        self.voices.retain_mut(|v| {
            let samples = &v.pcm.samples;
            let last = samples.len() - 1;
            for frame in out.chunks_exact_mut(2) {
                let i = v.pos as usize;
                if i >= last {
                    return false;
                }
                let frac = (v.pos - i as f64) as f32;
                let s = samples[i] + (samples[i + 1] - samples[i]) * frac;
                frame[0] += s * v.left;
                frame[1] += s * v.right;
                v.pos += v.step;
            }
            (v.pos as usize) < last
        });
        let master = self.master;
        for s in out.iter_mut() {
            *s = soft_clip(*s * master);
        }
    }
}

/// Transparent below `KNEE`, then rolls off smoothly towards +-1 instead of hard clipping
/// (dense songs routinely sum past full scale).
fn soft_clip(x: f32) -> f32 {
    const KNEE: f32 = 0.7;
    let a = x.abs();
    if a <= KNEE {
        x
    } else {
        let range = 1.0 - KNEE;
        x.signum() * (KNEE + range * ((a - KNEE) / range).tanh())
    }
}

/// A [`Mixer`] shared between the game thread (which triggers notes) and an audio
/// callback (which pulls samples). Implements [`AudioBackend`].
#[derive(Clone)]
pub struct SharedMixer(Arc<Mutex<Mixer>>);

impl SharedMixer {
    pub fn new(out_sample_rate: u32) -> Self {
        Self(Arc::new(Mutex::new(Mixer::new(out_sample_rate))))
    }

    /// Run `f` on the mixer; tolerates a poisoned lock so audio never takes the app down.
    pub fn with<R>(&self, f: impl FnOnce(&mut Mixer) -> R) -> R {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard)
    }

    pub fn mix_stereo(&self, out: &mut [f32]) {
        self.with(|m| m.mix_stereo(out));
    }
}

impl AudioBackend for SharedMixer {
    fn load_instrument(&mut self, instrument: &Instrument) {
        match instrument.decode() {
            Ok(pcm) => self.with(|m| m.set_instrument(instrument.id, Arc::new(pcm))),
            Err(e) => log::warn!("instrument {} skipped: {}", instrument.id, e),
        }
    }

    fn play(&mut self, note: &NoteBlock) {
        self.with(|m| m.play(note.instrument, note.frequency_ratio, note.volume, note.pan));
    }

    fn play_tick(&mut self, notes: &[NoteBlock]) {
        // One lock per tick instead of one per note.
        self.with(|m| {
            for n in notes {
                m.play(n.instrument, n.frequency_ratio, n.volume, n.pan);
            }
        });
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.with(|m| m.set_master_volume(volume));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(len: usize, rate: u32) -> Arc<Pcm> {
        Arc::new(Pcm {
            samples: (0..len).map(|i| i as f32 / len as f32).collect(),
            sample_rate: rate,
        })
    }

    #[test]
    fn plays_then_frees_voice() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, ramp(100, 1000));
        m.play(0, 1.0, 1.0, 0.0);
        let mut out = vec![0.0; 2 * 50];
        m.mix_stereo(&mut out);
        assert!(out.iter().any(|&s| s > 0.0));
        assert_eq!(m.active_voices(), 1);
        m.mix_stereo(&mut out);
        m.mix_stereo(&mut out);
        assert_eq!(m.active_voices(), 0);
    }

    #[test]
    fn double_ratio_plays_twice_as_fast() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, ramp(100, 1000));
        m.play(0, 2.0, 1.0, 0.0);
        let mut out = vec![0.0; 2 * 49];
        m.mix_stereo(&mut out);
        assert_eq!(m.active_voices(), 1);
        let mut out = vec![0.0; 2 * 2];
        m.mix_stereo(&mut out);
        assert_eq!(m.active_voices(), 0);
    }

    #[test]
    fn pan_is_balance() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, Arc::new(Pcm { samples: vec![0.5; 100], sample_rate: 1000 }));
        m.play(0, 1.0, 1.0, 1.0);
        let mut out = vec![0.0; 8];
        m.mix_stereo(&mut out);
        assert_eq!(out[0], 0.0);
        assert!(out[1] > 0.0);
    }

    #[test]
    fn steals_oldest_voice_when_full() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, ramp(1000, 1000));
        for _ in 0..m.max_voices + 10 {
            m.play(0, 1.0, 0.1, 0.0);
        }
        assert_eq!(m.active_voices(), m.max_voices);
    }

    #[test]
    fn unknown_instrument_and_bad_ratio_ignored() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, ramp(100, 1000));
        m.play(7, 1.0, 1.0, 0.0);
        m.play(0, f32::NAN, 1.0, 0.0);
        m.play(0, 0.0, 1.0, 0.0);
        assert_eq!(m.active_voices(), 0);
    }

    #[test]
    fn soft_clip_is_transparent_then_bounded() {
        assert_eq!(soft_clip(0.5), 0.5);
        assert_eq!(soft_clip(-0.7), -0.7);
        assert!(soft_clip(0.9) > 0.7 && soft_clip(0.9) < 0.9);
        assert!(soft_clip(50.0) <= 1.0 && soft_clip(-50.0) >= -1.0);
        assert!(soft_clip(1.0) < soft_clip(2.0));
    }

    #[test]
    fn output_is_clamped() {
        let mut m = Mixer::new(1000);
        m.set_instrument(0, Arc::new(Pcm { samples: vec![1.0; 100], sample_rate: 1000 }));
        for _ in 0..8 {
            m.play(0, 1.0, 1.0, 0.0);
        }
        let mut out = vec![0.0; 8];
        m.mix_stereo(&mut out);
        assert!(out.iter().all(|&s| s <= 1.0));
    }
}
