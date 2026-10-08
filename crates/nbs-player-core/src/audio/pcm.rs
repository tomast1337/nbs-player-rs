use lewton::inside_ogg::OggStreamReader;
use std::io::Cursor;

/// Decoded mono audio, normalised to -1..=1.
#[derive(Debug, Clone)]
pub struct Pcm {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub struct PcmError(pub String);

impl std::fmt::Display for PcmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ogg decode error: {}", self.0)
    }
}

impl std::error::Error for PcmError {}

impl Pcm {
    /// Decode an Ogg Vorbis stream, downmixing to mono.
    pub fn decode_ogg(bytes: &[u8]) -> Result<Pcm, PcmError> {
        let mut reader =
            OggStreamReader::new(Cursor::new(bytes)).map_err(|e| PcmError(e.to_string()))?;
        let channels = reader.ident_hdr.audio_channels.max(1) as usize;
        let sample_rate = reader.ident_hdr.audio_sample_rate;

        let mut samples = Vec::new();
        while let Some(packet) = reader
            .read_dec_packet_itl()
            .map_err(|e| PcmError(e.to_string()))?
        {
            for frame in packet.chunks_exact(channels) {
                let sum: f32 = frame.iter().map(|&s| s as f32).sum();
                samples.push(sum / channels as f32 / 32768.0);
            }
        }
        Ok(Pcm {
            samples,
            sample_rate,
        })
    }
}
