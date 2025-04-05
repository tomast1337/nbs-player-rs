use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use once_cell::sync::Lazy;
use raylib::prelude::*;

use crate::{app_state::AppState, note, utils};
#[derive(Debug, Clone)]
pub struct PianoProps {
    pub key_spacing: f32,
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
    pub font_size_white: f32,
    pub font_size_black: f32,
}

#[derive(Debug, Clone)]
pub struct PianoKey {
    pub key: u8,
    pub label: String,
    pub is_pressed: bool,
    pub white_key_index: Option<usize>,
    pub is_white: bool,
    pub press_offset: f32,
    pub press_velocity: f32,
    pub tints: Vec<(Color, f32)>,
}

const TINTS_SIZE: usize = 2;

impl PianoKey {
    fn new(key: u8, label: &str, is_white: bool, white_key_index: Option<usize>) -> Self {
        Self {
            key,
            label: label.to_string(),
            white_key_index: white_key_index,
            is_white,
            is_pressed: false,
            press_offset: 0.0,
            press_velocity: 0.0,
            tints: Vec::with_capacity(TINTS_SIZE),
        }
    }

    fn set_tint(&mut self, color: Color, weight: f32) {
        // Insert new tint at the front (top of the pile)
        self.tints.insert(0, (color, weight));

        // If we exceed the size, remove the oldest (bottom of the pile)
        if self.tints.len() > TINTS_SIZE {
            self.tints.pop(); // removes the last item
        }
    }
    pub fn press(&mut self, tint: Option<(&Color, f32)>) {
        self.is_pressed = true;
        self.press_offset = 0.0;
        self.press_velocity = 0.0;
        if let Some(tint) = tint {
            self.set_tint(tint.0.clone(), tint.1);
        }
    }
}

pub fn generate_piano_keys() -> (Vec<PianoKey>, HashMap<u8, usize>) {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    const START_MIDI: u8 = 21;
    const END_MIDI: u8 = 108;

    let mut white_labels = Vec::with_capacity(52);
    let mut black_labels = Vec::with_capacity(36);

    for midi in START_MIDI..=END_MIDI {
        let octave = (midi / 12) as i32 - 1;
        let name = format!("{}{}", NOTE_NAMES[(midi % 12) as usize], octave);
        let leaked: &'static str = Box::leak(name.into_boxed_str());

        if leaked.contains('#') {
            black_labels.push((leaked, midi));
        } else {
            white_labels.push((leaked, midi));
        }
    }

    assert_eq!(white_labels.len(), 52);
    assert_eq!(black_labels.len(), 36);

    let white_keys: Vec<PianoKey> = white_labels
        .iter()
        .enumerate()
        .map(|(index, &(label, midi))| PianoKey::new(midi, label, true, Some(index)))
        .collect();

    let black_keys: Vec<PianoKey> = black_labels
        .iter()
        .map(|&(label, midi)| {
            let pos = white_keys
                .iter()
                .position(|k| k.key > midi)
                .map(|i| i.saturating_sub(1));
            PianoKey::new(midi, label, false, pos)
        })
        .collect();

    let mut all_keys = white_keys.clone();
    all_keys.extend(black_keys);

    let key_map: HashMap<u8, usize> = all_keys
        .iter()
        .enumerate()
        .map(|(i, k)| (k.key, i))
        .collect();

    (all_keys, key_map)
}
const PRESS_FORCE: f32 = 50000000.0; // Press force
const DAMPING: f32 = 20.0; // Damping factor
const SPRING_CONSTANT: f32 = 700.0; // Spring constant
const MAX_OFFSET: f32 = 10.0; // Maximum offset
const MIN_OFFSET: f32 = -10.0; // Minimum offset
const STOP_THRESHOLD: f32 = 0.1; // Threshold to stop animation

pub fn update_key_animation(keys: &mut [PianoKey], delta_time: f32) {
    // Precompute all delta constants once
    let damping_delta = DAMPING * delta_time;
    let press_force_delta = PRESS_FORCE * delta_time;
    let spring_constant_delta = SPRING_CONSTANT * delta_time;

    // Process keys in cache-friendly linear order
    for key in keys.iter_mut() {
        // Load all fields into local variables first
        let is_pressed = key.is_pressed;
        let mut offset = key.press_offset;
        let mut velocity = key.press_velocity;

        // Calculate force
        let force = if is_pressed {
            -press_force_delta - damping_delta * (velocity + 1000.0)
        } else {
            -offset * spring_constant_delta - damping_delta * velocity
        };

        // Update physics
        velocity += force;
        offset += velocity * delta_time;

        // Apply constraints
        if is_pressed {
            if offset < MIN_OFFSET {
                offset = MIN_OFFSET;
                velocity = 0.0;
            }
        } else {
            if offset.abs() < STOP_THRESHOLD && velocity.abs() < STOP_THRESHOLD {
                offset = 0.0;
                velocity = 0.0;
            } else if offset > MAX_OFFSET {
                offset = MAX_OFFSET;
                velocity = 0.0;
            }
        }

        // Store back results
        key.press_offset = offset;
        key.press_velocity = velocity;
    }
}
pub fn draw_piano_keys(d: &mut RaylibDrawHandle<'_>, app_state: &AppState) {
    let piano_props = &app_state.piano_state.piano_props;
    let all_keys = &app_state.piano_state.all_keys;
    let key_texture = &app_state.textures.piano_key_texture;
    let font = &app_state.font;
    let window_width = app_state.window_width;
    let window_height = app_state.window_height;
    let theme = &app_state.theme;

    let key_spacing = piano_props.key_spacing;
    let white_key_width = piano_props.white_key_width;
    let white_key_height = piano_props.white_key_height;
    let black_key_width = piano_props.black_key_width;
    let black_key_height = piano_props.black_key_height;
    let font_size_white = piano_props.font_size_white;
    let font_size_black = piano_props.font_size_black;

    let total_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;
    let total_width = total_white_keys * (white_key_width + key_spacing) - key_spacing;

    let piano_x = (window_width - total_width) / 2.0;
    let piano_y = window_height - white_key_height;

    // Draw a background for the piano
    d.draw_rectangle_rec(
        Rectangle::new(piano_x, piano_y, total_width, white_key_height),
        Color::BLACK,
    );

    // Draw white keys first
    for key in all_keys.iter().filter(|k| k.is_white) {
        let width = white_key_width;
        let height = white_key_height;
        let x = piano_x + (key.white_key_index.unwrap() as f32 * (white_key_width + key_spacing));
        let y = piano_y - key.press_offset;

        let key_color = if key.press_offset != 0.0 && !key.tints.is_empty() {
            utils::blend_colors(theme.white_key_color, &key.tints)
        } else {
            theme.white_key_color
        };

        d.draw_texture_pro(
            &key_texture,
            Rectangle::new(
                0.0,
                0.0,
                key_texture.width as f32,
                key_texture.height as f32,
            ),
            Rectangle::new(x, y, white_key_width, white_key_height),
            Vector2::new(0.0, 0.0),
            0.0,
            key_color,
        );

        // Calculate font size to fit within the key
        let text_size = font.measure_text(&key.label, font_size_white, 0.);
        let text_width = text_size.x;
        let text_height = text_size.y;

        // Center text horizontally and vertically within the key
        let text_x = x + (width - text_width) / 2.0;
        // the bottom 4th
        let text_y = y + (height - text_height) / 2.0 + (height / 4.0);

        d.draw_text_pro(
            &font,
            &key.label,
            Vector2::new(text_x, text_y),
            Vector2::new(0.0, 0.0),
            0.0,
            font_size_white,
            0.,
            theme.white_text_key_color,
        );
    }

    // Draw black keys on top of white keys
    for key in all_keys.iter().filter(|k| !k.is_white) {
        if let Some(white_idx) = key.white_key_index {
            let width = black_key_width;
            let height = black_key_height;
            let x = piano_x + (white_idx as f32 + 0.5) * (white_key_width + key_spacing);
            let y = piano_y - 5.0 - key.press_offset;

            let key_color = if key.press_offset != 0.0 && !key.tints.is_empty() {
                utils::blend_colors(theme.black_key_color, &key.tints)
            } else {
                theme.black_key_color
            };

            d.draw_texture_pro(
                &key_texture,
                Rectangle::new(
                    0.0,
                    0.0,
                    key_texture.width as f32,
                    key_texture.height as f32,
                ),
                Rectangle::new(x, y, black_key_width, black_key_height),
                Vector2::new(0.0, 0.0),
                0.0,
                key_color,
            );

            // Calculate font size to fit within the key
            let text_size = font.measure_text(&key.label, font_size_black, 0.);
            let text_width = text_size.x;
            let text_height = text_size.y;

            // Center text horizontally and vertically within the key
            let text_x = x + (width - text_width) / 2.0;
            // the bottom 4th
            let text_y = y + (height - text_height) / 2.0 + (height / 4.0);

            d.draw_text_pro(
                &font,
                &key.label,
                Vector2::new(text_x, text_y),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size_black,
                0.,
                theme.black_text_key_color,
            );
        }
    }
}

fn calculate_font_size(key_width: f32, font: &Font, label_size: usize) -> f32 {
    // Constants
    const MAX_FONT_SIZE: f32 = 30.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const PADDING: f32 = 5.0;
    const PRECISION: f32 = 0.1;

    // Early exit for edge cases
    if label_size == 0 || key_width <= PADDING {
        return MIN_FONT_SIZE;
    }

    // Static cache for common label strings (1-4)
    static COMMON_LABELS: [&str; 4] = ["#", "##", "###", "####"];

    // Global cache for uncommon label sizes
    static STRING_CACHE: Lazy<Mutex<HashMap<usize, &'static str>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Get cached string or create new one
    let label = if label_size <= 4 {
        COMMON_LABELS[label_size - 1]
    } else {
        *STRING_CACHE
            .lock()
            .unwrap()
            .entry(label_size)
            .or_insert_with(|| {
                let s = "#".repeat(label_size);
                Box::leak(s.into_boxed_str())
            })
    };

    // Binary search for optimal font size
    let mut low = MIN_FONT_SIZE;
    let mut high = MAX_FONT_SIZE;
    let mut best_size = MIN_FONT_SIZE;
    let max_width = key_width - PADDING;

    while (high - low) > PRECISION {
        let mid = (low + high) / 2.0;
        let text_width = font.measure_text(label, mid, 0.0).x;

        if text_width <= max_width {
            best_size = mid;
            low = mid;
        } else {
            high = mid;
        }
    }

    best_size
}

pub fn initialize_piano_dimensions<'a>(
    window_width: f32,
    all_keys: &Vec<PianoKey>,
    font: &'a Font,
) -> PianoProps {
    let num_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;

    let black_key_width_ratio = 0.8;
    let black_key_height_ratio = 0.6;

    // Spacing between keys
    let key_spacing = 0.0;

    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = white_key_width * 3.0;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;

    let font_size_white = calculate_font_size(white_key_width, &font, 2);
    let font_size_black = calculate_font_size(black_key_width, &font, 3);

    PianoProps {
        key_spacing,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        font_size_white,
        font_size_black,
    }
}

#[derive(Debug, Clone)]
pub struct PianoState {
    pub all_keys: Vec<PianoKey>,
    pub key_map: HashMap<u8, usize>,
    pub piano_props: PianoProps,
}

// Cache for piano keys (generated once per application)
static PIANO_KEYS_CACHE: OnceLock<(Vec<PianoKey>, HashMap<u8, usize>)> = OnceLock::new();

impl PianoState {
    pub fn new(window_width: f32, font: &Font) -> Self {
        // Get or initialize cached piano keys
        let (all_keys, key_map) = PIANO_KEYS_CACHE
            .get_or_init(|| generate_piano_keys())
            .clone(); // Clone the cached data

        let piano_props = initialize_piano_dimensions(window_width, &all_keys, font);

        PianoState {
            all_keys,
            key_map,
            piano_props,
        }
    }

    pub fn reset_keys(&mut self) {
        // Use iter_mut for direct mutable access
        self.all_keys.iter_mut().for_each(|key| {
            key.is_pressed = false;
        });
    }

    pub fn trigger_key_press(
        &mut self,
        note_blocks: &[note::NoteBlock],
        note_color_map: &HashMap<u32, Color>,
    ) {
        const VOLUME_DIVISOR: f32 = 5.0;

        note_blocks.iter().for_each(|note| {
            if let Some(&key_index) = self.key_map.get(&note.key) {
                // Dereference color immediately to avoid holding reference
                let color = *note_color_map
                    .get(&note.instrument)
                    .unwrap_or(&Color::WHITE);

                self.all_keys[key_index].press(Some((&color, note.volume / VOLUME_DIVISOR)));
            }
        });
    }

    pub fn update_key_animation(&mut self, delta_time: f32) {
        // Pass the slice directly for better cache locality
        update_key_animation(self.all_keys.as_mut_slice(), delta_time);
    }
}
