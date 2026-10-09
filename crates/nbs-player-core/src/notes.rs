use std::collections::HashMap;

use crate::piano::{PianoKey, PianoProps};
use crate::types::{Rect, Rgba, TextMeasure, Vec2};
use crate::utils;

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub instrument: u32,
    pub key: u8,
    /// Playback speed relative to the instrument's tuned pitch.
    pub frequency_ratio: f32,
    /// 0..=1
    pub volume: f32,
    /// -1 (left) ..= 1 (right)
    pub pan: f32,
}

/// Converts a NBS file into one `Vec<NoteBlock>` per tick.
/// `base_keys` maps instrument id to the key its sample is tuned to.
pub fn get_note_blocks(
    song: &nbs_rs::NbsFile,
    base_keys: &HashMap<u32, f64>,
) -> Vec<Vec<NoteBlock>> {
    const INV_12: f32 = 1.0 / 12.0;
    let song_length = song.header.song_length as usize;
    let average_notes_per_tick = if song_length > 0 {
        (song.notes.len() / song_length).max(1)
    } else {
        1
    };

    let mut note_blocks: Vec<Vec<NoteBlock>> = (0..song_length)
        .map(|_| Vec::with_capacity(average_notes_per_tick))
        .collect();

    for note in &song.notes {
        let tick = note.tick as usize;
        if tick >= note_blocks.len() {
            continue;
        }
        let Some(&tone) = base_keys.get(&(note.instrument as u32)) else {
            continue;
        };
        let key = note.key as f32;
        let pitch = note.pitch as f32;
        note_blocks[tick].push(NoteBlock {
            instrument: note.instrument as u32,
            key: note.key,
            frequency_ratio: utils::fast_pow2((key + (pitch / 100.0) - tone as f32) * INV_12),
            volume: note.velocity as f32 / 100.0,
            pan: (note.panning as f32 / 100.0).clamp(-1.0, 1.0),
        });
    }

    if note_blocks.iter().all(Vec::is_empty) {
        log::warn!("No note blocks loaded");
    } else {
        log::info!("Loaded note blocks");
    }
    note_blocks
}

/// Instrument id -> note color. The first 16 are the classic NBS colors, the rest are
/// spread around the HSV hue wheel.
pub fn generate_instrument_palette() -> HashMap<u32, Rgba> {
    const BASE: [(u32, &str); 16] = [
        (0, "1964ac"),
        (1, "3c8e48"),
        (2, "be6b6b"),
        (3, "bebe19"),
        (4, "9d5a98"),
        (5, "572b21"),
        (6, "bec65c"),
        (7, "be19be"),
        (8, "52908d"),
        (9, "bebebe"),
        (10, "1991be"),
        (11, "be2328"),
        (12, "be5728"),
        (13, "19be19"),
        (14, "be1957"),
        (15, "575757"),
    ];
    const ALPHA: u8 = (255.0 * 0.90) as u8;
    const HUE_STEP: f32 = 3.6; // 360° / 100 colors

    let mut colors = HashMap::with_capacity(116);
    for &(id, hex) in &BASE {
        let color = Rgba::from_hex(hex).unwrap_or(Rgba::WHITE);
        colors.insert(id, Rgba { a: ALPHA, ..color });
    }
    for i in 0..100 {
        let color = Rgba::from_hsv(i as f32 * HUE_STEP, 1.0, 1.0);
        colors.insert((i + 16) as u32, Rgba { a: ALPHA, ..color });
    }
    colors
}

/// Everything needed to lay out the falling notes for one frame.
pub struct NoteScene<'a> {
    pub window_width: f32,
    pub window_height: f32,
    pub keys: &'a [PianoKey],
    pub key_map: &'a HashMap<u8, usize>,
    pub piano_props: &'a PianoProps,
    pub note_blocks: &'a [Vec<NoteBlock>],
    pub current_tick: f32,
    pub note_dim: f32,
    pub key_spacing: f32,
    pub instrument_colors: &'a HashMap<u32, Rgba>,
    pub font_size_2: f32,
    pub font_size_3: f32,
}

/// One note block ready to draw: a textured quad plus its centered label.
pub struct NoteSprite<'a> {
    pub rect: Rect,
    pub color: Rgba,
    pub label: &'a str,
    pub font_size: f32,
    pub label_pos: Vec2,
}

/// Calls `f` for every note currently on screen, oldest tick first.
/// Returns how many were emitted.
pub fn visible_notes<'a>(
    scene: &NoteScene<'a>,
    measure: &dyn TextMeasure,
    mut f: impl FnMut(NoteSprite<'a>),
) -> usize {
    let NoteScene {
        window_width,
        window_height,
        keys,
        key_map,
        piano_props,
        note_blocks,
        current_tick,
        note_dim,
        key_spacing,
        instrument_colors,
        font_size_2,
        font_size_3,
    } = *scene;

    let sliding_window_size = (window_height / note_dim) as i32 + 2;
    let window_start_tick = (current_tick - sliding_window_size as f32).max(0.0) as i32;
    let window_end_tick = current_tick as i32 + sliding_window_size;

    let half_window_width = window_width / 2.0;
    let half_note_dim = note_dim / 2.0;
    let base_offset = -half_window_width + half_note_dim;
    let min_y = 0.0;
    let max_y = window_height - piano_props.white_key_height;

    let mut emitted = 0;
    for tick in window_start_tick as usize..window_end_tick as usize {
        let Some(notes) = note_blocks.get(tick) else {
            continue;
        };
        let tick_f32 = tick as f32;
        for note in notes {
            let Some(&key_index) = key_map.get(&note.key) else {
                continue;
            };
            let piano_key = &keys[key_index];

            let x_pos = if piano_key.is_white {
                key_index as f32 * (note_dim + key_spacing) + base_offset
            } else if let Some(white_idx) = piano_key.white_key_index {
                (white_idx as f32 + 0.5) * (note_dim + key_spacing) + base_offset
            } else {
                continue;
            };

            let y_pos = window_height - ((tick_f32 - current_tick) * note_dim) - note_dim;
            if !(y_pos + note_dim > min_y && y_pos < max_y) {
                continue;
            }

            // NOTE: `alpha` takes 0..=1 and clamps, so this is effectively always opaque.
            // Kept as-is to preserve the historical look.
            let color = instrument_colors
                .get(&note.instrument)
                .copied()
                .unwrap_or(Rgba::WHITE)
                .alpha((note.volume * 205.0) + 50.);

            let label = piano_key.label.as_str();
            let font_size = if label.len() > 2 {
                font_size_3
            } else {
                font_size_2
            };
            let text_dim = measure.measure_text(label, font_size);

            f(NoteSprite {
                rect: Rect::new(
                    x_pos + half_window_width - half_note_dim,
                    y_pos,
                    note_dim,
                    note_dim,
                ),
                color,
                label,
                font_size,
                // -0.5 matches the sub-pixel origin the label was historically drawn with.
                label_pos: Vec2::new(
                    x_pos + half_window_width - text_dim.x / 2.0 - 0.5,
                    y_pos + half_note_dim - text_dim.y / 2.0 - 0.5,
                ),
            });
            emitted += 1;
        }
    }
    emitted
}

/// Largest font size in 8..=30 whose `label_size`-char label fits a note block.
fn calculate_font_size(note_dim: f32, measure: &dyn TextMeasure, label_size: usize) -> f32 {
    const MAX_FONT_SIZE: f32 = 30.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const PADDING: f32 = 5.0;

    if label_size == 0 {
        return MIN_FONT_SIZE;
    }
    let test_string = "#".repeat(label_size.min(4));

    let (mut low, mut high, mut best) = (MIN_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE);
    while (high - low) > 0.1 {
        let mid = (low + high) / 2.0;
        if measure.measure_text(&test_string, mid).x <= note_dim - PADDING {
            best = mid;
            low = mid;
        } else {
            high = mid;
        }
    }
    best
}

/// `(font_size_3, font_size_2)`. The second size is fitted against a 4-char string, so
/// plain 2-char labels end up smaller than the 3-char ones (historical behaviour).
pub fn calculate_note_block_font_sizes(note_dim: f32, measure: &dyn TextMeasure) -> (f32, f32) {
    (
        calculate_font_size(note_dim, measure, 3),
        calculate_font_size(note_dim, measure, 4),
    )
}
