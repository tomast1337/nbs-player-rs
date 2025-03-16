use std::collections::HashMap;

use macroquad::{
    color,
    math::Vec2,
    text::{TextParams, draw_text_ex, measure_text},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

#[derive(Debug)]
pub struct PianoProps {
    pub key_spacing: f32,
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
    pub white_key_texture: Texture2D,
    pub black_key_texture: Texture2D,
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
    const PRESS_FORCE: f32 = 500000.; // Press force
    const DAMPING: f32 = 20.; // Damping factor
    const SPRING_CONSTANT: f32 = 700.; // Spring constant
    const MAX_OFFSET: f32 = 5.0; // Maximum offset
    const MIN_OFFSET: f32 = -5.0; // Minimum offset

    for key in keys.iter_mut() {
        if key.is_pressed {
            let force = -PRESS_FORCE - DAMPING * (key.press_velocity + 1000.);
            key.press_velocity += force * delta_time;
            key.press_offset += key.press_velocity * delta_time;

            if key.press_offset < MIN_OFFSET {
                key.press_offset = MIN_OFFSET;
                key.press_velocity = 0.0;
            }
        } else {
            let force = -key.press_offset * SPRING_CONSTANT - DAMPING * key.press_velocity;
            key.press_velocity += force * delta_time;
            key.press_offset += key.press_velocity * delta_time;

            if key.press_offset.abs() < 0.1 && key.press_velocity.abs() < 0.1 {
                key.press_offset = 0.0;
                key.press_velocity = 0.0;
            }

            if key.press_offset > MAX_OFFSET {
                key.press_offset = MAX_OFFSET;
                key.press_velocity = 0.0;
            }
        }
    }
}

pub fn draw_piano_keys(
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<PianoKey>,
    piano_props: &PianoProps,
) {
    let key_spacing = piano_props.key_spacing;
    let white_key_width = piano_props.white_key_width;
    let white_key_height = piano_props.white_key_height;
    let black_key_width = piano_props.black_key_width;
    let black_key_height = piano_props.black_key_height;
    let white_key_texture = &piano_props.white_key_texture;
    let black_key_texture = &piano_props.black_key_texture;

    let total_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;
    let total_width = total_white_keys * (white_key_width + key_spacing) - key_spacing;

    for (i, key) in all_keys.iter().enumerate() {
        let (x_pos, y_pos, width, height, texture, text_color) = if key.is_white {
            let x =
                (i as f32 * (white_key_width + key_spacing)) + (window_width - total_width) / 2.0;
            let y = window_height - white_key_height - key.press_offset;
            (
                x,
                y,
                white_key_width,
                white_key_height,
                white_key_texture,
                color::BLACK,
            )
        } else if let Some(white_idx) = key.white_key_index {
            let x = (white_idx as f32 + 0.5) * (white_key_width + key_spacing);
            let y = window_height - 5. - white_key_height - key.press_offset;
            (
                x,
                y,
                black_key_width,
                black_key_height,
                black_key_texture,
                color::WHITE,
            )
        } else {
            continue;
        };

        // Draw key with texture
        draw_texture_ex(
            texture,
            x_pos,
            y_pos,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(width, height)),
                ..Default::default()
            },
        );

        let font = crate::FONT.get().unwrap();

        // Calculate font size to fit within the key
        let max_font_size = 18; // Maximum font size
        let min_font_size = 8; // Minimum font size
        let mut font_size = max_font_size;

        // Measure text width and adjust font size if necessary
        let mut text_width = measure_text(&key.label, Some(&font), font_size, 1.0).width;
        while text_width > width - 5. && font_size > min_font_size {
            font_size -= 1;
            text_width = measure_text(&key.label, Some(&font), font_size, 1.0).width;
        }

        // Center text horizontally within the key
        let text_x = x_pos + (width - text_width) / 2.0;

        // Draw label
        draw_text_ex(
            &key.label,
            text_x,
            y_pos + height - height * 0.25,
            TextParams {
                font: Some(&font.clone()),
                font_size,
                color: text_color,
                ..Default::default()
            },
        );
    }
}

pub fn initialize_piano_dimensions(
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<PianoKey>,
) -> PianoProps {
    let (white_key_texture, black_key_texture) = load_piano_key_textures();

    let num_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;

    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 0.8;
    let black_key_height_ratio = 0.6;

    // Spacing between keys
    let key_spacing = 0.0;

    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = window_height * key_size_relative_to_screen;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;
    PianoProps {
        key_spacing,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        white_key_texture,
        black_key_texture,
    }
}

pub fn load_piano_key_textures() -> (Texture2D, Texture2D) {
    let (key_black_bytes, key_white_bytes) = (
        include_bytes!("../assets/textures/key_black.png"),
        include_bytes!("../assets/textures/key_white.png"),
    );
    let (key_black_image, key_white_image) = (
        Texture2D::from_file_with_format(key_black_bytes, None),
        Texture2D::from_file_with_format(key_white_bytes, None),
    );

    (key_white_image, key_black_image)
}
