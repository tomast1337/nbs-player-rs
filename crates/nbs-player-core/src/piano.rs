use std::collections::HashMap;

use crate::notes::NoteBlock;
use crate::theme::Theme;
use crate::types::{Rect, Rgba, TextMeasure, Vec2};
use crate::utils;

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
    pub tints: Vec<(Rgba, f32)>,
}

const TINTS_SIZE: usize = 2;

impl PianoKey {
    fn new(key: u8, label: &str, is_white: bool, white_key_index: Option<usize>) -> Self {
        Self {
            key,
            label: label.to_string(),
            white_key_index,
            is_white,
            is_pressed: false,
            press_offset: 0.0,
            press_velocity: 0.0,
            tints: Vec::with_capacity(TINTS_SIZE),
        }
    }

    fn set_tint(&mut self, color: Rgba, weight: f32) {
        // Newest tint goes on top; drop the oldest beyond TINTS_SIZE.
        self.tints.insert(0, (color, weight));
        self.tints.truncate(TINTS_SIZE);
    }

    pub fn press(&mut self, tint: Option<(Rgba, f32)>) {
        self.is_pressed = true;
        self.press_offset = 0.0;
        self.press_velocity = 0.0;
        if let Some((color, weight)) = tint {
            self.set_tint(color, weight);
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
        if name.contains('#') {
            black_labels.push((name, midi));
        } else {
            white_labels.push((name, midi));
        }
    }

    assert_eq!(white_labels.len(), 52);
    assert_eq!(black_labels.len(), 36);

    let white_keys: Vec<PianoKey> = white_labels
        .iter()
        .enumerate()
        .map(|(index, (label, midi))| PianoKey::new(*midi, label, true, Some(index)))
        .collect();

    let black_keys: Vec<PianoKey> = black_labels
        .iter()
        .map(|(label, midi)| {
            let pos = white_keys
                .iter()
                .position(|k| k.key > *midi)
                .map(|i| i.saturating_sub(1));
            PianoKey::new(*midi, label, false, pos)
        })
        .collect();

    let mut all_keys = white_keys;
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
    let damping_delta = DAMPING * delta_time;
    let press_force_delta = PRESS_FORCE * delta_time;
    let spring_constant_delta = SPRING_CONSTANT * delta_time;

    for key in keys.iter_mut() {
        let is_pressed = key.is_pressed;
        let mut offset = key.press_offset;
        let mut velocity = key.press_velocity;

        let force = if is_pressed {
            -press_force_delta - damping_delta * (velocity + 1000.0)
        } else {
            -offset * spring_constant_delta - damping_delta * velocity
        };

        velocity += force;
        offset += velocity * delta_time;

        if is_pressed {
            if offset < MIN_OFFSET {
                offset = MIN_OFFSET;
                velocity = 0.0;
            }
        } else if offset.abs() < STOP_THRESHOLD && velocity.abs() < STOP_THRESHOLD {
            offset = 0.0;
            velocity = 0.0;
        } else if offset > MAX_OFFSET {
            offset = MAX_OFFSET;
            velocity = 0.0;
        }

        key.press_offset = offset;
        key.press_velocity = velocity;
    }
}

/// Largest font size in 8..=30 whose `label_size`-char label fits `key_width`.
fn calculate_font_size(key_width: f32, measure: &dyn TextMeasure, label_size: usize) -> f32 {
    const MAX_FONT_SIZE: f32 = 30.0;
    const MIN_FONT_SIZE: f32 = 8.0;
    const PADDING: f32 = 5.0;
    const PRECISION: f32 = 0.1;

    if label_size == 0 || key_width <= PADDING {
        return MIN_FONT_SIZE;
    }
    let label = "#".repeat(label_size);
    let max_width = key_width - PADDING;

    let (mut low, mut high, mut best) = (MIN_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE);
    while (high - low) > PRECISION {
        let mid = (low + high) / 2.0;
        if measure.measure_text(&label, mid).x <= max_width {
            best = mid;
            low = mid;
        } else {
            high = mid;
        }
    }
    best
}

pub fn initialize_piano_dimensions(
    window_width: f32,
    all_keys: &[PianoKey],
    measure: &dyn TextMeasure,
) -> PianoProps {
    let num_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;

    let black_key_width_ratio = 0.8;
    let black_key_height_ratio = 0.6;
    let key_spacing = 0.0;

    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = white_key_width * 3.0;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;

    PianoProps {
        key_spacing,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        font_size_white: calculate_font_size(white_key_width, measure, 2),
        font_size_black: calculate_font_size(black_key_width, measure, 3),
    }
}

#[derive(Debug, Clone)]
pub struct PianoState {
    pub all_keys: Vec<PianoKey>,
    pub key_map: HashMap<u8, usize>,
    pub piano_props: PianoProps,
}

impl PianoState {
    pub fn new(window_width: f32, measure: &dyn TextMeasure) -> Self {
        let (all_keys, key_map) = generate_piano_keys();
        let piano_props = initialize_piano_dimensions(window_width, &all_keys, measure);
        PianoState {
            all_keys,
            key_map,
            piano_props,
        }
    }

    /// Recompute key sizes after a window width change.
    pub fn resize(&mut self, window_width: f32, measure: &dyn TextMeasure) {
        self.piano_props = initialize_piano_dimensions(window_width, &self.all_keys, measure);
    }

    pub fn reset_keys(&mut self) {
        self.all_keys.iter_mut().for_each(|key| key.is_pressed = false);
    }

    pub fn trigger_key_press(
        &mut self,
        note_blocks: &[NoteBlock],
        note_color_map: &HashMap<u32, Rgba>,
    ) {
        const VOLUME_DIVISOR: f32 = 5.0;

        for note in note_blocks {
            if let Some(&key_index) = self.key_map.get(&note.key) {
                let color = note_color_map
                    .get(&note.instrument)
                    .copied()
                    .unwrap_or(Rgba::WHITE);
                self.all_keys[key_index].press(Some((color, note.volume / VOLUME_DIVISOR)));
            }
        }
    }

    pub fn update_key_animation(&mut self, delta_time: f32) {
        update_key_animation(self.all_keys.as_mut_slice(), delta_time);
    }

    /// Black backdrop rectangle behind the keys.
    pub fn bounds(&self, window_width: f32, window_height: f32) -> Rect {
        let p = &self.piano_props;
        let total_white = self.all_keys.iter().filter(|k| k.is_white).count() as f32;
        let total_width = total_white * (p.white_key_width + p.key_spacing) - p.key_spacing;
        Rect::new(
            (window_width - total_width) / 2.0,
            window_height - p.white_key_height,
            total_width,
            p.white_key_height,
        )
    }
}

/// One piano key ready to draw: a textured quad plus its label.
pub struct KeySprite<'a> {
    pub rect: Rect,
    pub color: Rgba,
    pub label: &'a str,
    pub label_pos: Vec2,
    pub font_size: f32,
    pub text_color: Rgba,
}

/// Calls `f` for every key, all white keys first and then the black keys on top.
pub fn for_each_key_sprite<'a>(
    state: &'a PianoState,
    window_width: f32,
    window_height: f32,
    theme: &Theme,
    measure: &dyn TextMeasure,
    mut f: impl FnMut(KeySprite<'a>),
) {
    let p = &state.piano_props;
    let bounds = state.bounds(window_width, window_height);
    let (piano_x, piano_y) = (bounds.x, bounds.y);

    let label_pos = |label: &str, font_size: f32, rect: Rect| -> Vec2 {
        let text = measure.measure_text(label, font_size);
        Vec2::new(
            rect.x + (rect.w - text.x) / 2.0,
            // lower quarter of the key
            rect.y + (rect.h - text.y) / 2.0 + (rect.h / 4.0),
        )
    };

    for key in state.all_keys.iter().filter(|k| k.is_white) {
        let Some(idx) = key.white_key_index else { continue };
        let rect = Rect::new(
            piano_x + idx as f32 * (p.white_key_width + p.key_spacing),
            piano_y - key.press_offset,
            p.white_key_width,
            p.white_key_height,
        );
        let color = if key.press_offset != 0.0 && !key.tints.is_empty() {
            utils::blend_colors(theme.white_key_color, &key.tints)
        } else {
            theme.white_key_color
        };
        f(KeySprite {
            rect,
            color,
            label: &key.label,
            label_pos: label_pos(&key.label, p.font_size_white, rect),
            font_size: p.font_size_white,
            text_color: theme.white_text_key_color,
        });
    }

    for key in state.all_keys.iter().filter(|k| !k.is_white) {
        let Some(white_idx) = key.white_key_index else { continue };
        let rect = Rect::new(
            piano_x + (white_idx as f32 + 0.5) * (p.white_key_width + p.key_spacing),
            piano_y - 5.0 - key.press_offset,
            p.black_key_width,
            p.black_key_height,
        );
        let color = if key.press_offset != 0.0 && !key.tints.is_empty() {
            utils::blend_colors(theme.black_key_color, &key.tints)
        } else {
            theme.black_key_color
        }
        .alpha(1.);
        f(KeySprite {
            rect,
            color,
            label: &key.label,
            label_pos: label_pos(&key.label, p.font_size_black, rect),
            font_size: p.font_size_black,
            text_color: theme.black_text_key_color,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed;
    impl TextMeasure for Fixed {
        fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
            Vec2::new(text.len() as f32 * font_size * 0.5, font_size)
        }
    }

    #[test]
    fn keyboard_layout() {
        let (keys, map) = generate_piano_keys();
        assert_eq!(keys.len(), 88);
        assert_eq!(map.len(), 88);
        assert_eq!(keys[map[&21]].label, "A0");
        assert_eq!(keys[map[&108]].label, "C8");
        assert!(keys.iter().filter(|k| !k.is_white).all(|k| k.white_key_index.is_some()));
    }

    #[test]
    fn press_springs_back() {
        let mut state = PianoState::new(1280.0, &Fixed);
        let idx = state.key_map[&60];
        state.all_keys[idx].press(None);
        for _ in 0..30 {
            state.update_key_animation(1.0 / 60.0);
        }
        assert!(state.all_keys[idx].press_offset < 0.0);
        state.reset_keys();
        for _ in 0..300 {
            state.update_key_animation(1.0 / 60.0);
        }
        assert_eq!(state.all_keys[idx].press_offset, 0.0);
    }

    #[test]
    fn emits_every_key_once() {
        let state = PianoState::new(1280.0, &Fixed);
        let mut n = 0;
        for_each_key_sprite(&state, 1280.0, 720.0, &Theme::default(), &Fixed, |_| n += 1);
        assert_eq!(n, 88);
    }
}
