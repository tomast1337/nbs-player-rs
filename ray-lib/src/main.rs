use std::collections::HashMap;

use raylib::prelude::*;
use song::load_nbs_file;

mod note;
mod piano;
mod song;

fn main() {
    let window_width: f32 = 1280.;
    let window_height = 720.;

    let nbs_file = load_nbs_file(None);

    let song_name = String::from_utf8(nbs_file.header.song_name.clone()).unwrap();
    let song_author = String::from_utf8(nbs_file.header.song_author.clone()).unwrap();

    let title = format!("{} - {}", song_name, song_author);

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title(title.as_str())
        .build();

    let (mut all_keys, key_map) = piano::generate_piano_keys();

    let piano_props = piano::initialize_piano_dimensions(
        window_width,
        window_height,
        &all_keys,
        &mut rl,
        &thread,
    );

    let note_blocks = note::get_note_blocks(&nbs_file);

    let note_texture = load_note_texture(&mut rl, &thread);

    load_sound_assets(None);

    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time = 0.0; // Elapsed time in seconds

    let notes_per_second = 10.0; // Adjust based on song tempo
    let note_dim = piano_props.white_key_width;
    let key_spacing = piano_props.key_spacing; // Spacing between keys

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);
        elapsed_time += delta_time;
        // show fps counter right top corner
        d.draw_fps(window_width as i32 - 100, 10);

        // Update current tick based on elapsed time and tempo
        current_tick = elapsed_time * notes_per_second;

        for key in &mut all_keys {
            key.is_pressed = false;
        }

        if let Some(notes) = note_blocks.get(current_tick as usize) {
            for note in notes {
                if let Some(&key_index) = key_map.get(&note.key) {
                    // Trigger the corresponding piano key
                    all_keys[key_index].is_pressed = true;
                }
            }
        }

        // Draw notes
        let window_start_tick = (current_tick - 100.0).max(0.0) as i32; // Start of sliding window
        let window_end_tick = (current_tick as i32) + 100; // End of sliding window

        for tick in window_start_tick..window_end_tick {
            let tick = tick as f32; // Convert tick to f32 for calculations
            if let Some(notes) = note_blocks.get(tick as usize) {
                for note in notes {
                    if let Some(&key_index) = key_map.get(&note.key) {
                        let piano_key = &all_keys[key_index];

                        // Calculate note position
                        let x_pos = if piano_key.is_white {
                            // White key positioning
                            key_index as f32 * (note_dim + key_spacing) - window_width / 2.0
                                + note_dim / 2.0
                        } else if let Some(white_idx) = piano_key.white_key_index {
                            // Black key positioning
                            (white_idx as f32 + 0.5) * (note_dim + key_spacing) - window_width / 2.0
                                + note_dim / 2.0
                        } else {
                            continue;
                        };

                        let y_pos = window_height
                            - ((tick as f32 - current_tick as f32) * (note_dim))
                            - note_dim;

                        // Check if the note is visible on the screen
                        if y_pos + note_dim > 0.0 && y_pos < window_height {
                            let note_rect = Rectangle::new(
                                x_pos + window_width / 2.0 - note_dim / 2.0,
                                y_pos,
                                note_dim,
                                note_dim,
                            );

                            d.draw_texture_pro(
                                &note_texture,
                                Rectangle::new(
                                    0.0,
                                    0.0,
                                    note_texture.width as f32,
                                    note_texture.height as f32,
                                ),
                                note_rect,
                                Vector2::new(0.0, 0.0),
                                0.0,
                                Color::WHITE,
                            );

                            // Draw the tone (note name) on the note
                            let text = &piano_key.label;
                            let text_width = d.measure_text(text, 10);
                            let text_x = note_rect.x + (note_rect.width - text_width as f32) / 2.0;
                            let text_y = note_rect.y + (note_rect.height - 10.0) / 2.0;

                            d.draw_text(text, text_x as i32, text_y as i32, 10, Color::WHITE);
                        }
                    }
                }
            }
        }
        d.clear_background(Color::DARKGRAY);
        d.draw_text(&title, 12, 12, 20, Color::BLACK);
        piano::update_key_animation(&mut all_keys, delta_time);
        piano::draw_piano_keys(window_width, window_height, &all_keys, &piano_props, d);
    }
}

fn load_sound_assets(extra_sounds: Option<Vec<String>>) -> HashMap<u32, &'static [u8]> {
    println!("{:?}", extra_sounds);
    let mut sounds_data = HashMap::new();

    sounds_data.insert(0, include_bytes!("../assets/bass.ogg") as &[u8]);
    sounds_data.insert(1, include_bytes!("../assets/bd.ogg") as &[u8]);
    sounds_data.insert(2, include_bytes!("../assets/harp.ogg") as &[u8]);
    sounds_data.insert(3, include_bytes!("../assets/snare.ogg") as &[u8]);
    sounds_data.insert(4, include_bytes!("../assets/hat.ogg") as &[u8]);
    sounds_data.insert(5, include_bytes!("../assets/guitar.ogg") as &[u8]);
    sounds_data.insert(6, include_bytes!("../assets/flute.ogg") as &[u8]);
    sounds_data.insert(7, include_bytes!("../assets/bell.ogg") as &[u8]);
    sounds_data.insert(8, include_bytes!("../assets/icechime.ogg") as &[u8]);
    sounds_data.insert(9, include_bytes!("../assets/xylobone.ogg") as &[u8]);
    sounds_data.insert(10, include_bytes!("../assets/iron_xylophone.ogg") as &[u8]);
    sounds_data.insert(11, include_bytes!("../assets/cow_bell.ogg") as &[u8]);
    sounds_data.insert(12, include_bytes!("../assets/didgeridoo.ogg") as &[u8]);
    sounds_data.insert(13, include_bytes!("../assets/bit.ogg") as &[u8]);
    sounds_data.insert(14, include_bytes!("../assets/banjo.ogg") as &[u8]);
    sounds_data.insert(15, include_bytes!("../assets/pling.ogg") as &[u8]);

    sounds_data
}

fn load_note_texture(rl: &mut RaylibHandle, thread: &RaylibThread) -> Texture2D {
    let note_image_bytes = include_bytes!("../assets/note_block.png");
    let note_image = Image::load_image_from_mem(".png", note_image_bytes).unwrap();
    let note_texture = rl.load_texture_from_image(thread, &note_image).unwrap();
    note_texture
}
