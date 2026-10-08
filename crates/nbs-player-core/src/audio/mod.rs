//! Audio abstraction: instrument bank, ogg decoding, a pure-software mixer and the
//! [`AudioBackend`] trait that output implementations (raylib, cpal, ...) satisfy.

mod instruments;
mod mixer;
mod pcm;

pub use instruments::{Instrument, InstrumentBank};
pub use mixer::{Mixer, SharedMixer};
pub use pcm::{Pcm, PcmError};

use crate::notes::NoteBlock;

/// Something that can turn note events into sound.
pub trait AudioBackend {
    /// Register (or replace) an instrument. Called once per instrument before playback.
    fn load_instrument(&mut self, instrument: &Instrument);

    /// Start one note. `frequency_ratio` is relative to the instrument's tuned pitch,
    /// `volume` is 0..=1, `pan` is -1 (left) ..= 1 (right).
    fn play(&mut self, note: &NoteBlock);

    fn play_tick(&mut self, notes: &[NoteBlock]) {
        for note in notes {
            self.play(note);
        }
    }

    /// Master volume 0..=1.
    fn set_master_volume(&mut self, volume: f32);
}
