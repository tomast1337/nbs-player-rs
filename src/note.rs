use std::collections::HashMap;

use macroquad::{
    color::{self, Color},
    math::{Rect, Vec2},
    text::{TextParams, draw_text_ex, measure_text},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};
use nbs_rs;

use crate::piano;

pub fn load_note_texture() -> Texture2D {
    let note_image_bytes = include_bytes!("../assets/textures/note_block.png");
    let note_texture = Texture2D::from_file_with_format(note_image_bytes, None);
    note_texture
}

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
            note_blocks[tick].push(NoteBlock {
                was_played: false,
                instrument: note.instrument,
                key: note.key,
                velocity: note.velocity,
                panning: note.panning,
                pitch: note.pitch,
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

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let (r, g, b) = (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    );

    (r, g, b)
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
        let hex = u32::from_str_radix(color_str, 16).unwrap();
        let mut color = Color::from_hex(hex);
        color.a = 0.90;
        instrument_colors.insert(*id, color);
    }

    let initial_palette_size = instrument_colors.len();

    // add 100 more colors to the palette following hue wheel
    for i in 0..100 {
        let h = i as f64 * 3.6; // Spread colors evenly on the hue wheel
        let s = 1.0;
        let l = 0.5;

        let (r, g, b) = hsl_to_rgb(h, s, l);

        let color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.90);

        instrument_colors.insert((i + initial_palette_size) as u8, color);
    }

    instrument_colors
}

pub fn draw_notes(
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
) -> i32 {
    let sliding_window_size = (window_height / note_dim) as i32 + 2;
    let window_start_tick = (current_tick - sliding_window_size as f32).max(0.0) as i32;
    let window_end_tick = (current_tick as i32) + sliding_window_size;

    let base_offset = -window_width / 2. + note_dim / 2.;
    let min_y = 0.;
    let max_y = window_height - piano_props.white_key_height;

    // Count notes being rendered
    let mut notes_rendered = 0;

    let font = crate::FONT.get().unwrap();

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
                        let note_rect = Rect::new(
                            x_pos + window_width / 2.0 - note_dim / 2.0,
                            y_pos,
                            note_dim,
                            note_dim,
                        );

                        // Get note color by the instrument index
                        let mut color = match instrument_colors.get(&note.instrument) {
                            Some(&color) => color,
                            None => color::WHITE,
                        }
                        .clone();

                        color.a = 0.90;

                        // Draw the note texture
                        draw_texture_ex(
                            note_texture,
                            note_rect.x,
                            note_rect.y,
                            color,
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(note_rect.w as f32, note_rect.h as f32)),
                                ..Default::default()
                            },
                        );

                        // Draw the tone (note name) on the note
                        let text = &piano_key.label;

                        // Calculate font size to fit within the note block
                        let max_font_size = 20; // Maximum font size
                        let min_font_size = 8; // Minimum font size
                        let mut font_size = max_font_size;

                        // Measure text width and height
                        let mut text_width = measure_text(text, Some(&font), font_size, 1.0).width;
                        let mut text_height =
                            measure_text(text, Some(&font), font_size, 1.0).height;

                        // Adjust font size if the text is too large
                        while (text_width > note_rect.w - 5. || text_height > note_rect.h - 5.)
                            && font_size > min_font_size
                        {
                            font_size -= 1;
                            text_width = measure_text(text, Some(&font), font_size, 1.0).width;
                            text_height = measure_text(text, Some(&font), font_size, 1.0).height;
                        }

                        // Center text horizontally and vertically within the note block
                        let text_x = note_rect.x + (note_rect.w - text_width) / 2.0;
                        let text_y = note_rect.y + (note_rect.h) / 2.0;

                        // Draw the text
                        draw_text_ex(
                            text,
                            text_x,
                            text_y,
                            TextParams {
                                font: Some(&font),
                                font_size,
                                color: color::WHITE,
                                ..Default::default()
                            },
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
