use std::{cell::RefCell, collections::HashMap};

use nbs_rs;
use raylib::prelude::*;

use crate::{app_state::AppState, audio::AudioClip, utils};

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub instrument: u32,
    pub key: u8,
    pub frequency_ratio: f32,
    pub volume: f32,
    pub pan: f32,
}

/// Converts a NBS file into a vector of note blocks
pub fn get_note_blocks(
    song: &nbs_rs::NbsFile,
    sounds: &HashMap<u32, AudioClip>,
) -> Vec<Vec<NoteBlock>> {
    const INV_12: f32 = 1.0 / 12.0;
    // Pre allocate the ticks so it doesn't have to resize the on each iteration
    let song_length = song.header.song_length as usize;
    let total_notes = song.notes.len();
    let average_notes_per_tick = if song_length > 0 {
        (total_notes / song_length).max(1) // Ensure at least 1 note per tick
    } else {
        1
    };

    let mut note_blocks: Vec<Vec<NoteBlock>> = (0..song_length)
        .map(|_| Vec::with_capacity(average_notes_per_tick))
        .collect();

    for note in &song.notes {
        let tick = note.tick as usize;
        if tick < note_blocks.len() {
            // get note layer
            let sound_id = note.instrument as u32;
            let key = note.key as f32;
            let velocity = note.velocity as f32;
            let panning = note.panning as f32;
            let pitch = note.pitch as f32;

            let sound_data = match sounds.get(&sound_id) {
                Some(data) => data,
                None => continue,
            };

            // Calculate sound properties
            let tone = sound_data.pitch as f32;
            let frequency_ratio = utils::fast_pow2((key + (pitch / 100.0) - tone) * INV_12);
            let volume = velocity / 100.0;
            let pan = ((panning + 100.0) / 200.0) - 0.5;
            note_blocks[tick].push(NoteBlock {
                instrument: note.instrument as u32,
                key: note.key,
                frequency_ratio,
                volume,
                pan,
            });
        }
    }

    if !note_blocks.iter().all(Vec::is_empty) {
        log::info!("Loaded note blocks");
    } else {
        log::warn!("No note blocks loaded");
    }

    note_blocks
}

/// Generates a color palette for instruments
/// Returns a HashMap with instrument IDs as keys and Color as values
/// The first 16 colors are predefined, and the rest are generated using HSV color space
pub fn generate_instrument_palette() -> HashMap<u32, Color> {
    // Pre-allocate the HashMap with the expected capacity (16 base colors + 100 generated)
    let mut instrument_colors = HashMap::with_capacity(116);

    // Static array instead of Vec for compile-time optimization
    const INSTRUMENT_COLOR_PALETTE: [(u32, &str); 16] = [
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

    // Precompute alpha to avoid repeated casting
    const ALPHA: u8 = (255.0 * 0.90) as u8;

    // Process the base colors
    for &(id, color_str) in &INSTRUMENT_COLOR_PALETTE {
        let mut color = match Color::from_hex(color_str) {
            Ok(c) => c,
            Err(_) => {
                log::warn!("Invalid color string: {}", color_str);
                Color::WHITE // Fallback to white if the color string is invalid
            }
        };
        color.a = ALPHA;
        instrument_colors.insert(id, color);
    }

    // Precompute the hue step to avoid division in the loop
    const HUE_STEP: f32 = 3.6; // 360° / 100 colors

    // Generate additional colors (100 more)
    for i in 0..100 {
        let hue = i as f32 * HUE_STEP;
        let mut color = Color::color_from_hsv(hue, 1.0, 1.0);
        color.a = ALPHA;
        instrument_colors.insert((i + 16) as u32, color);
    }

    instrument_colors
}

/// Draws the notes on the screen
/// Returns the number of notes rendered
pub fn draw_notes(d: &mut RaylibDrawHandle<'_>, app_state: &AppState) -> i32 {
    // Extract all needed values from app_state first
    let window_width = app_state.window_width;
    let window_height = app_state.window_height;
    let all_keys = &app_state.piano_state.all_keys;
    let key_map = &app_state.piano_state.key_map;
    let note_blocks = &app_state.song_state.note_blocks;
    let piano_props = &app_state.piano_state.piano_props;
    let note_texture = &app_state.textures.note_texture;
    let current_tick = app_state.song_state.current_tick;
    let note_dim = app_state.song_state.note_dim;
    let key_spacing = app_state.song_state.key_spacing;
    let instrument_colors = &app_state.song_state.instrument_colors;
    let font = &app_state.font;
    let font_size_2 = app_state.song_state.font_size_2;
    let font_size_3 = app_state.song_state.font_size_3;

    // Precompute values that don't change during the loop
    let sliding_window_size = (window_height / note_dim) as i32 + 2;
    let window_start_tick = (current_tick - sliding_window_size as f32).max(0.0) as i32;
    let window_end_tick = current_tick as i32 + sliding_window_size;

    let half_window_width = window_width / 2.0;
    let half_note_dim = note_dim / 2.0;
    let base_offset = -half_window_width + half_note_dim;
    let min_y = 0.0;
    let max_y = window_height - piano_props.white_key_height;

    // Precompute texture source rectangle (doesn't change)
    let note_source_rec = Rectangle::new(
        0.0,
        0.0,
        note_texture.width as f32,
        note_texture.height as f32,
    );

    // Count notes being rendered
    let mut notes_rendered = 0;

    for tick in window_start_tick as usize..window_end_tick as usize {
        let tick_f32 = tick as f32;
        if let Some(notes) = note_blocks.get(tick) {
            for note in notes {
                if let Some(&key_index) = key_map.get(&note.key) {
                    let piano_key = &all_keys[key_index];

                    // Calculate note position
                    let x_pos = if piano_key.is_white {
                        key_index as f32 * (note_dim + key_spacing) + base_offset
                    } else if let Some(white_idx) = piano_key.white_key_index {
                        (white_idx as f32 + 0.5) * (note_dim + key_spacing) + base_offset
                    } else {
                        continue;
                    };

                    let y_pos = window_height - ((tick_f32 - current_tick) * note_dim) - note_dim;

                    // Check if the note is visible on the screen
                    if y_pos + note_dim > min_y && y_pos < max_y {
                        let note_rect = Rectangle::new(
                            x_pos + half_window_width - half_note_dim,
                            y_pos,
                            note_dim,
                            note_dim,
                        );

                        // Get note color by the instrument index
                        let mut color = instrument_colors
                            .get(&note.instrument)
                            .copied()
                            .unwrap_or(Color::WHITE);

                        // convet note.velocity  0-100 to 50-255
                        color = color.alpha((note.volume * 205.0) + 50.);

                        // Draw the note texture
                        d.draw_texture_pro(
                            note_texture,
                            note_source_rec,
                            note_rect,
                            Vector2::zero(),
                            0.0,
                            color,
                        );

                        // Draw the tone (note name) on the note
                        let text = &piano_key.label;
                        let font_size = if text.len() > 2 {
                            font_size_3
                        } else {
                            font_size_2
                        };
                        let text_dim = font.measure_text(text, font_size, 0.0);
                        let text_x = x_pos + half_window_width - text_dim.x / 2.0;
                        let text_y = y_pos + half_note_dim - text_dim.y / 2.0;

                        d.draw_text_pro(
                            font,
                            text,
                            Vector2::new(text_x, text_y),
                            Vector2::new(0.5, 0.5),
                            0.0,
                            font_size,
                            0.0,
                            Color::WHITE,
                        );

                        // Increment notes rendered count
                        notes_rendered += 1;
                    }
                }
            }
        }
    }
    notes_rendered
}

fn calculate_font_size(note_dim: f32, font: &Font, label_size: usize) -> f32 {
    const MAX_FONT_SIZE: f32 = 30.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const PADDING: f32 = 5.0;

    if label_size == 0 {
        return MIN_FONT_SIZE;
    }

    // Use a single allocation for all test strings
    static TEST_STRINGS: once_cell::sync::Lazy<Vec<String>> =
        once_cell::sync::Lazy::new(|| (0..=4).map(|i| "#".repeat(i)).collect());

    let test_string = &TEST_STRINGS.get(label_size).unwrap_or(&TEST_STRINGS[4]);

    // Binary search with early exit for perfect fits
    let mut low = MIN_FONT_SIZE;
    let mut high = MAX_FONT_SIZE;
    let mut best_size = MIN_FONT_SIZE;

    while (high - low) > 0.1 {
        // Precision threshold
        let mid = (low + high) / 2.0;
        let text_width = font.measure_text(test_string, mid, 0.0).x;

        if text_width <= note_dim - PADDING {
            best_size = mid;
            low = mid;
        } else {
            high = mid;
        }
    }

    best_size
}

thread_local! {
    static FONT_SIZE_CACHE: RefCell<HashMap< u32, (f32, f32)>> = RefCell::new(HashMap::new());
}

pub fn calculate_note_block_font_sizes(note_dim: f32, font: &Font) -> (f32, f32) {
    // Create a hash key from the note dimension and font properties
    let cache_key = note_dim.to_bits();

    FONT_SIZE_CACHE.with(|cache| {
        if let Some(sizes) = cache.borrow().get(&cache_key) {
            return *sizes;
        }

        // Calculate fresh values
        let font_size_3 = calculate_font_size(note_dim, font, 3);
        let font_size_4 = calculate_font_size(note_dim, font, 4);
        let result = (font_size_3, font_size_4);

        cache.borrow_mut().insert(cache_key, result);
        result
    })
}
