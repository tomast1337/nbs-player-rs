use std::collections::HashMap;

use crate::theme::Theme;
use raylib::prelude::*;
#[derive(Debug)]
pub struct PianoProps<'a> {
    pub key_spacing: f32,
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
    pub font: &'a Font,
    pub font_size_white: f32,
    pub font_size_black: f32,
}

#[derive(Clone, Debug)]
pub struct PianoKey {
    pub key: u8,
    pub label: String,
    pub is_pressed: bool,
    pub white_key_index: Option<usize>,
    pub is_white: bool,
    pub press_offset: f32,
    pub press_velocity: f32,
}

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
        }
    }
}

pub fn generate_piano_keys() -> (Vec<PianoKey>, HashMap<u8, usize>) {
    let white_keys: [(&str, i32); 52] = [
        ("A0", 21),
        ("B0", 23),
        ("C1", 24),
        ("D1", 26),
        ("E1", 28),
        ("F1", 29),
        ("G1", 31),
        ("A1", 33),
        ("B1", 35),
        ("C2", 36),
        ("D2", 38),
        ("E2", 40),
        ("F2", 41),
        ("G2", 43),
        ("A2", 45),
        ("B2", 47),
        ("C3", 48),
        ("D3", 50),
        ("E3", 52),
        ("F3", 53),
        ("G3", 55),
        ("A3", 57),
        ("B3", 59),
        ("C4", 60),
        ("D4", 62),
        ("E4", 64),
        ("F4", 65),
        ("G4", 67),
        ("A4", 69),
        ("B4", 71),
        ("C5", 72),
        ("D5", 74),
        ("E5", 76),
        ("F5", 77),
        ("G5", 79),
        ("A5", 81),
        ("B5", 83),
        ("C6", 84),
        ("D6", 86),
        ("E6", 88),
        ("F6", 89),
        ("G6", 91),
        ("A6", 93),
        ("B6", 95),
        ("C7", 96),
        ("D7", 98),
        ("E7", 100),
        ("F7", 101),
        ("G7", 103),
        ("A7", 105),
        ("B7", 107),
        ("C8", 108),
    ];

    let black_keys: [(&str, i32); 36] = [
        ("A#0", 22),
        ("C#1", 25),
        ("D#1", 27),
        ("F#1", 30),
        ("G#1", 32),
        ("A#1", 34),
        ("C#2", 37),
        ("D#2", 39),
        ("F#2", 42),
        ("G#2", 44),
        ("A#2", 46),
        ("C#3", 49),
        ("D#3", 51),
        ("F#3", 54),
        ("G#3", 56),
        ("A#3", 58),
        ("C#4", 61),
        ("D#4", 63),
        ("F#4", 66),
        ("G#4", 68),
        ("A#4", 70),
        ("C#5", 73),
        ("D#5", 75),
        ("F#5", 78),
        ("G#5", 80),
        ("A#5", 82),
        ("C#6", 85),
        ("D#6", 87),
        ("F#6", 90),
        ("G#6", 92),
        ("A#6", 94),
        ("C#7", 97),
        ("D#7", 99),
        ("F#7", 102),
        ("G#7", 104),
        ("A#7", 106),
    ];

    let white_keys_vec: Vec<PianoKey> = white_keys
        .iter()
        .enumerate()
        .map(|(index, (label, key))| PianoKey::new(*key as u8, label, true, Some(index)))
        .collect();

    let black_keys_vec: Vec<PianoKey> = black_keys
        .iter()
        .map(|(label, key)| {
            let white_key_index = white_keys_vec
                .iter()
                .position(|white_key| white_key.key > *key as u8)
                .map(|index| index.saturating_sub(1));

            PianoKey::new(*key as u8, label, false, white_key_index)
        })
        .collect();

    // Combine into single vector
    let mut all_keys = white_keys_vec;
    all_keys.extend(black_keys_vec);

    // Create hashmap for quick lookup
    let mut key_map = HashMap::new();
    for (idx, key) in all_keys.iter().enumerate() {
        key_map.insert(key.key, idx);
    }

    (all_keys, key_map)
}

pub fn update_key_animation(keys: &mut [PianoKey], delta_time: f32) {
    const PRESS_FORCE: f32 = 50000000.0; // Press force
    const DAMPING: f32 = 20.0; // Damping factor
    const SPRING_CONSTANT: f32 = 700.0; // Spring constant
    const MAX_OFFSET: f32 = 10.0; // Maximum offset
    const MIN_OFFSET: f32 = -10.0; // Minimum offset
    const STOP_THRESHOLD: f32 = 0.1; // Threshold to stop animation

    // Precompute constants
    let damping_delta = DAMPING * delta_time;
    let press_force_delta = PRESS_FORCE * delta_time;
    let spring_constant_delta = SPRING_CONSTANT * delta_time;

    for key in keys.iter_mut() {
        let force = if key.is_pressed {
            // Force when key is pressed
            -press_force_delta - damping_delta * (key.press_velocity + 1000.0)
        } else {
            // Force when key is released
            -key.press_offset * spring_constant_delta - damping_delta * key.press_velocity
        };

        // Update velocity and offset
        key.press_velocity += force;
        key.press_offset += key.press_velocity * delta_time;

        // Clamp offset and handle stopping condition
        if key.is_pressed {
            if key.press_offset < MIN_OFFSET {
                key.press_offset = MIN_OFFSET;
                key.press_velocity = 0.0;
            }
        } else {
            if key.press_offset.abs() < STOP_THRESHOLD && key.press_velocity.abs() < STOP_THRESHOLD
            {
                key.press_offset = 0.0;
                key.press_velocity = 0.0;
            } else if key.press_offset > MAX_OFFSET {
                key.press_offset = MAX_OFFSET;
                key.press_velocity = 0.0;
            }
        }
    }
}

pub fn draw_piano_keys(
    d: &mut RaylibDrawHandle<'_>,
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<PianoKey>,
    piano_props: &PianoProps,
    key_texture: &Texture2D,
    theme: &Theme,
) {
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

    let font = piano_props.font;

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

        d.draw_texture_pro(
            key_texture,
            Rectangle::new(
                0.0,
                0.0,
                key_texture.width as f32,
                key_texture.height as f32,
            ),
            Rectangle::new(x, y, white_key_width, white_key_height),
            Vector2::new(0.0, 0.0),
            0.0,
            theme.white_key_color,
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

            d.draw_texture_pro(
                key_texture,
                Rectangle::new(
                    0.0,
                    0.0,
                    key_texture.width as f32,
                    key_texture.height as f32,
                ),
                Rectangle::new(x, y, black_key_width, black_key_height),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.black_key_color,
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
    let max_font_size = 30.;
    let mut font_size = max_font_size;
    // Maximum font size
    let min_font_size = 8.;
    // Minimum font size

    // generate a string with the same length as the note label
    let label = "#".repeat(label_size);

    // Measure text width and height
    let mut text_width = font.measure_text(&label, font_size, 0.).x;

    // Adjust font size if the text is too large
    while (text_width > key_width - 5.) && font_size > min_font_size {
        font_size -= 1.;
        text_width = font.measure_text(&label, font_size, 0.).x;
    }
    font_size
}

pub fn initialize_piano_dimensions<'a>(
    window_width: f32,
    all_keys: &Vec<PianoKey>,
    font: &'a Font,
) -> PianoProps<'a> {
    let num_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;

    let black_key_width_ratio = 0.8;
    let black_key_height_ratio = 0.6;

    // Spacing between keys
    let key_spacing = 0.0;

    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = white_key_width * 3.0;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;

    let font_size_white = calculate_font_size(white_key_width, font, 2);
    let font_size_black = calculate_font_size(black_key_width, font, 3);

    PianoProps {
        key_spacing,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        font,
        font_size_white,
        font_size_black,
    }
}
