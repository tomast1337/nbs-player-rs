use std::collections::HashMap;

use nbs_rs;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibDrawHandle},
    texture::Texture2D,
};

use crate::piano;

pub fn load_note_texture(
    rl: &mut raylib::RaylibHandle,
    thread: &raylib::RaylibThread,
) -> raylib::texture::Texture2D {
    let note_image_bytes = include_bytes!("../assets/note_block.png");
    let note_image = raylib::texture::Image::load_image_from_mem(".png", note_image_bytes).unwrap();
    let note_texture = rl.load_texture_from_image(thread, &note_image).unwrap();
    note_texture
}

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub was_played: bool,
    pub tick: u16,
    pub layer: u16,
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
                tick: note.tick,
                layer: note.layer,
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

pub fn generate_instrument_palette() -> HashMap<u8, &'static str> {
    let mut instrument_colors = HashMap::new();
    let instrument_color_palette: [(u8, &str); 16] = [
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
        let color = &color[1..];
        instrument_colors.insert(*id, color);
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
    d: &mut RaylibDrawHandle<'_>,
    instrument_colors: &HashMap<u8, &str>,
) -> i32 {
    let sliding_window_size = (window_height / note_dim) as i32 + 2;
    let window_start_tick = (current_tick - sliding_window_size as f32).max(0.0) as i32;
    let window_end_tick = (current_tick as i32) + sliding_window_size;

    let texture_source_rect = Rectangle::new(
        0.0,
        0.0,
        note_texture.width as f32,
        note_texture.height as f32,
    );

    let base_offset = -window_width / 2.0 + note_dim / 2.0;
    let min_y = 0.0;
    let max_y = window_height - piano_props.white_key_height;

    // Count notes being rendered
    let mut notes_rendered = 0;

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

                        // get note color by the instrument index if larger than the array size, Color::WHITE is used
                        let color = match instrument_colors.get(&note.instrument) {
                            Some(&color) => Color::from_hex(color),
                            None => Ok(Color::WHITE),
                        }
                        .unwrap();

                        d.draw_texture_pro(
                            note_texture,
                            texture_source_rect,
                            note_rect,
                            Vector2::new(0.0, 0.0),
                            0.0,
                            color,
                        );

                        // Draw the tone (note name) on the note
                        let text = &piano_key.label;
                        let text_width = d.measure_text(text, 10);
                        let text_x = note_rect.x + (note_rect.width - text_width as f32) / 2.0;
                        let text_y = note_rect.y + (note_rect.height - 10.0) / 2.0;

                        d.draw_text(text, text_x as i32, text_y as i32, 10, Color::WHITE);

                        // Increment notes rendered count
                        notes_rendered += 1;
                    }
                }
            }
        }
    }
    notes_rendered
}
