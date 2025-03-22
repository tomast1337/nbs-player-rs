use std::collections::HashMap;

use nbs_rs;
use raylib::prelude::*;

use crate::piano;

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub was_played: bool,
    pub instrument: u8,
    pub key: u8,
    pub velocity: u8,
    pub panning: i8,
    pub pitch: i16,
}

pub fn get_note_blocks(song: &nbs_rs::NbsFile) -> Vec<Vec<NoteBlock>> {
    // Pre allocate the ticks so it doesn't have to resize the on each iteration
    let mut note_blocks: Vec<Vec<NoteBlock>> = vec![Vec::new(); song.header.song_length as usize];

    for note in &song.notes {
        let tick = note.tick as usize;
        if tick < note_blocks.len() {
            // get note layer
            if note.layer as usize >= song.layers.len() {
                note_blocks[tick].push(NoteBlock {
                    was_played: false,
                    instrument: note.instrument,
                    key: note.key,
                    velocity: note.velocity,
                    panning: note.panning,
                    pitch: note.pitch,
                });
            } else {
                let layer = &song.layers[note.layer as usize];
                note_blocks[tick].push(NoteBlock {
                    was_played: false,
                    instrument: note.instrument,
                    key: note.key,
                    velocity: note.velocity * layer.volume,
                    panning: note.panning * layer.panning,
                    pitch: note.pitch,
                });
            }
        }
    }

    if !note_blocks.iter().all(Vec::is_empty) {
        log::info!("Loaded note blocks");
    } else {
        log::warn!("No note blocks loaded");
    }

    note_blocks
}

pub fn generate_instrument_palette() -> HashMap<u8, Color> {
    let mut instrument_colors = HashMap::new();

    let instrument_color_palette = vec![
        (0, "#1964ac"),
        (1, "#3c8e48"),
        (2, "#be6b6b"),
        (3, "#bebe19"),
        (4, "#9d5a98"),
        (5, "#572b21"),
        (6, "#bec65c"),
        (7, "#be19be"),
        (8, "#52908d"),
        (9, "#bebebe"),
        (10, "#1991be"),
        (11, "#be2328"),
        (12, "#be5728"),
        (13, "#19be19"),
        (14, "#be1957"),
        (15, "#575757"),
    ];

    for (id, color) in instrument_color_palette.iter() {
        // remove the # from the color string
        let color_str = &color[1..];
        let mut color = Color::from_hex(color_str).unwrap();
        color.a = (255. * 0.90) as u8; // Set alpha to 0.90
        instrument_colors.insert(*id, color);
    }

    let initial_palette_size = instrument_colors.len();

    // add 100 more colors to the palette following hue wheel
    for i in 0..100 {
        let hue = i as f32 * 3.6; // Spread colors evenly on the hue wheel
        let saturation = 1.0;
        let value = 1.0;

        let color = Color::color_from_hsv(hue, saturation, value);
        instrument_colors.insert((i + initial_palette_size) as u8, color);
    }

    instrument_colors
}

pub fn draw_notes(
    d: &mut RaylibDrawHandle<'_>,
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<piano::PianoKey>,
    key_map: &HashMap<u8, usize>,
    note_blocks: &Vec<Vec<NoteBlock>>,
    piano_props: &piano::PianoProps,
    note_texture: &Texture2D,
    current_tick: f32,
    note_dim: f32,
    key_spacing: f32,
    instrument_colors: &HashMap<u8, Color>,
    font: &Font,
) -> i32 {
    let sliding_window_size = (window_height / note_dim) as i32 + 2;
    let window_start_tick = (current_tick - sliding_window_size as f32).max(0.0) as i32;
    let window_end_tick = (current_tick as i32) + sliding_window_size;

    let base_offset = -window_width / 2. + note_dim / 2.;
    let min_y = 0.;
    let max_y = window_height - piano_props.white_key_height;

    // Count notes being rendered
    let mut notes_rendered = 0;

    // Calculate font size to fit within the note block
    let font_size_3 = calculate_font_size(note_dim, font, 3);
    let font_size_2 = calculate_font_size(note_dim, font, 4);

    for tick in window_start_tick as usize..window_end_tick as usize {
        let tick_f32 = tick as f32;
        if let Some(notes) = note_blocks.get(tick as usize) {
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
                            x_pos + window_width / 2.0 - note_dim / 2.0,
                            y_pos,
                            note_dim,
                            note_dim,
                        );

                        // Get note color by the instrument index
                        let mut color = match instrument_colors.get(&note.instrument) {
                            Some(&color) => color,
                            None => Color::WHITE,
                        }
                        .clone();

                        // convet note.velocity  0-100 to 0-255
                        color =
                            color.alpha(((note.velocity as f32 / 100.0) * 255.0).round() as f32);

                        // Draw the note texture
                        d.draw_texture_pro(
                            note_texture,
                            Rectangle::new(
                                0.0,
                                0.0,
                                note_texture.width as f32,
                                note_texture.height as f32,
                            ),
                            Rectangle::new(
                                note_rect.x,
                                note_rect.y,
                                note_rect.width,
                                note_rect.height,
                            ),
                            Vector2::new(0.0, 0.0),
                            0.0,
                            color,
                        );

                        // Draw the tone (note name) on the note
                        let text = &piano_key.label;

                        // Center text horizontally and vertically within the note block
                        let font_size = if text.len() > 2 {
                            font_size_3
                        } else {
                            font_size_2
                        };
                        let text_dim = font.measure_text(text, font_size, 0.);
                        let text_x = x_pos + window_width / 2.0 - text_dim.x / 2.;
                        let text_y = y_pos + note_dim / 2.0 - text_dim.y / 2.;
                        // Draw the text
                        //d.draw_text(text, text_x, text_y, font_size, Color::WHITE);
                        d.draw_text_pro(
                            &font,
                            text,
                            Vector2::new(text_x as f32, text_y as f32),
                            Vector2::new(0.5, 0.5),
                            0.0,
                            font_size,
                            0.,
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
    while (text_width > note_dim - 5.) && font_size > min_font_size {
        font_size -= 1.;
        text_width = font.measure_text(&label, font_size, 0.).x;
    }
    font_size
}
