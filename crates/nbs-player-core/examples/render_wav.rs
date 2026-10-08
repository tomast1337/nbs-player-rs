//! Renders a song offline through the software mixer: no window, no audio device.
//!
//! cargo run --release -p nbs-player-core --example render_wav -- song.nbs out.wav [seconds]

use nbs_player_core::audio::{AudioBackend, InstrumentBank, SharedMixer};
use nbs_player_core::{notes, song};

const RATE: u32 = 44_100;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("usage: render_wav <song.nbs|.zip> <out.wav> [seconds]");
    let output = args.next().expect("missing output path");
    let max_seconds: Option<f32> = args.next().and_then(|s| s.parse().ok());

    let bytes = std::fs::read(&input).expect("cannot read song");
    let data = song::load_song_bytes(&bytes).expect("cannot load song");
    let bank = InstrumentBank::new(&data.extra_sounds);
    let blocks = notes::get_note_blocks(&data.song, &bank.base_keys());

    let mut mixer = SharedMixer::new(RATE);
    bank.install(&mut mixer);

    // header.tempo is ticks per second * 100
    let tick_seconds = 100.0 / data.song.header.tempo as f32;
    let total = blocks.len() as f32 * tick_seconds;
    let seconds = max_seconds.map_or(total, |m| m.min(total));
    let total_frames = (seconds * RATE as f32) as usize + RATE as usize * 2; // 2s tail

    let mut out = Vec::<i16>::with_capacity(total_frames * 2);
    let mut buf = vec![0.0f32; 2 * 512];
    let (mut peak, mut sum_sq, mut clipped) = (0.0f32, 0.0f64, 0usize);
    let mut next_tick = 0usize;

    while out.len() < total_frames * 2 {
        let frame_start = out.len() / 2;
        // fire every tick that falls inside this block
        let block_end_time = (frame_start + 512) as f32 / RATE as f32;
        while next_tick < blocks.len()
            && (next_tick as f32 * tick_seconds) < block_end_time
            && (next_tick as f32 * tick_seconds) < seconds
        {
            mixer.play_tick(&blocks[next_tick]);
            next_tick += 1;
        }
        mixer.mix_stereo(&mut buf);
        for &s in &buf {
            peak = peak.max(s.abs());
            sum_sq += (s as f64) * (s as f64);
            if s.abs() >= 0.98 {
                clipped += 1;
            }
            out.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
    }

    write_wav(&output, &out);
    println!(
        "{:.1}s rendered, peak {:.3}, rms {:.4}, samples >= 0.98 {} ({:.3}%)",
        out.len() as f32 / 2.0 / RATE as f32,
        peak,
        (sum_sq / out.len() as f64).sqrt(),
        clipped,
        clipped as f64 * 100.0 / out.len() as f64
    );
}

fn write_wav(path: &str, samples: &[i16]) {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&2u16.to_le_bytes()); // stereo
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, b).expect("cannot write wav");
}
