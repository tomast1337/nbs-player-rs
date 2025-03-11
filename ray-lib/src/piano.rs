use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PianoKey {
    pub key: u8,
    pub label: String,
    pub is_pressed: bool,
    pub white_key_index: Option<usize>,
    pub is_white: bool, // New field to distinguish key type
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
        .map(|(index, (label, key))| PianoKey {
            key: *key as u8,
            label: label.to_string(),
            is_pressed: false,
            white_key_index: Some(index),
            is_white: true,
        })
        .collect();

    let black_keys_vec: Vec<PianoKey> = black_keys
        .iter()
        .map(|(label, key)| {
            let white_key_index = white_keys_vec
                .iter()
                .position(|white_key| white_key.key > *key as u8)
                .map(|index| index.saturating_sub(1));

            PianoKey {
                key: *key as u8,
                label: label.to_string(),
                is_pressed: false,
                white_key_index,
                is_white: false,
            }
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
