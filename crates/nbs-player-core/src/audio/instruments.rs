use std::collections::HashMap;
use std::sync::Arc;

use super::{AudioBackend, Pcm, PcmError};
use crate::song::ExtraSound;

/// Vanilla Minecraft note block samples, ids 0..16 (all tuned to F#3 = key 45).
const DEFAULT_CLIPS: [&[u8]; 16] = [
    include_bytes!("../../../../assets/sounds/harp.ogg"),
    include_bytes!("../../../../assets/sounds/bass.ogg"),
    include_bytes!("../../../../assets/sounds/bd.ogg"),
    include_bytes!("../../../../assets/sounds/snare.ogg"),
    include_bytes!("../../../../assets/sounds/hat.ogg"),
    include_bytes!("../../../../assets/sounds/guitar.ogg"),
    include_bytes!("../../../../assets/sounds/flute.ogg"),
    include_bytes!("../../../../assets/sounds/bell.ogg"),
    include_bytes!("../../../../assets/sounds/icechime.ogg"),
    include_bytes!("../../../../assets/sounds/xylobone.ogg"),
    include_bytes!("../../../../assets/sounds/iron_xylophone.ogg"),
    include_bytes!("../../../../assets/sounds/cow_bell.ogg"),
    include_bytes!("../../../../assets/sounds/didgeridoo.ogg"),
    include_bytes!("../../../../assets/sounds/bit.ogg"),
    include_bytes!("../../../../assets/sounds/banjo.ogg"),
    include_bytes!("../../../../assets/sounds/pling.ogg"),
];

const DEFAULT_BASE_KEY: f64 = 45.0;

#[derive(Debug, Clone)]
pub struct Instrument {
    pub id: u32,
    /// Key the sample is tuned to; notes are pitched relative to it.
    pub base_key: f64,
    /// Encoded Ogg Vorbis data.
    pub ogg: Arc<[u8]>,
}

impl Instrument {
    pub fn decode(&self) -> Result<Pcm, PcmError> {
        Pcm::decode_ogg(&self.ogg)
    }
}

/// Default instruments followed by any custom ones from the song archive.
#[derive(Debug, Clone)]
pub struct InstrumentBank {
    pub instruments: Vec<Instrument>,
}

impl InstrumentBank {
    pub fn new(extra: &[ExtraSound]) -> Self {
        let mut instruments: Vec<Instrument> = DEFAULT_CLIPS
            .iter()
            .map(|clip| (Arc::<[u8]>::from(*clip), DEFAULT_BASE_KEY))
            .chain(
                extra
                    .iter()
                    .map(|e| (Arc::<[u8]>::from(e.ogg.as_slice()), e.key)),
            )
            .enumerate()
            .map(|(i, (ogg, base_key))| Instrument {
                id: i as u32,
                base_key,
                ogg,
            })
            .collect();
        instruments.shrink_to_fit();
        log::info!("Instrument bank: {} instruments", instruments.len());
        Self { instruments }
    }

    /// Instrument id -> tuned base key, used when converting notes to frequency ratios.
    pub fn base_keys(&self) -> HashMap<u32, f64> {
        self.instruments.iter().map(|i| (i.id, i.base_key)).collect()
    }

    pub fn install(&self, backend: &mut dyn AudioBackend) {
        for instrument in &self.instruments {
            backend.load_instrument(instrument);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_default_clips_decode() {
        let bank = InstrumentBank::new(&[]);
        assert_eq!(bank.instruments.len(), 16);
        for i in &bank.instruments {
            let pcm = i.decode().unwrap_or_else(|e| panic!("instrument {}: {e}", i.id));
            assert!(!pcm.samples.is_empty(), "instrument {} empty", i.id);
            assert!(pcm.sample_rate > 0);
        }
    }
}
