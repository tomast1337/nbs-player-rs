use raylib::prelude::*;
use song::load_nbs_file;

mod note;
mod piano;
mod song;

fn main() {
    let window_width = 1280.;
    let window_height = 720.;

    let nbs_file = load_nbs_file(None);

    let song_name = String::from_utf8(nbs_file.header.song_name.clone()).unwrap();
    let song_author = String::from_utf8(nbs_file.header.song_author.clone()).unwrap();

    let title = format!("{} - {}", song_name, song_author);

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title(title.as_str())
        .build();

    let (all_keys, key_map) = piano::generate_piano_keys();

    let piano_props = piano::initialize_piano_dimensions(window_width, window_height, &all_keys);

    let note_blocks = note::get_note_blocks(&nbs_file);

    let note_image_bytes = include_bytes!("../assets/note_block.png");
    let note_image = Image::load_image_from_mem(".png", note_image_bytes).unwrap();
    let note_texture = rl.load_texture_from_image(&thread, &note_image).unwrap();

    let notes_per_second = 10.0; // Adjust based on song tempo
    let note_dim = piano_props.white_key_width;
    let key_spacing = piano_props.key_spacing; // Spacing between keys

    let mut current_tick; // Current tick in the song
    let mut elapsed_time = 0.0; // Elapsed time in seconds

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);
        elapsed_time += delta_time;
        // show fps counter right top corner
        d.draw_fps(window_width as i32 - 100, 10);

        // Update current tick based on elapsed time and tempo
        current_tick = (elapsed_time * notes_per_second) as usize;

        // Draw notes
        let window_start_tick = current_tick.saturating_sub(100); // Start of sliding window
        let window_end_tick = current_tick + 100; // End of sliding window

        for tick in window_start_tick..window_end_tick {
            if let Some(notes) = note_blocks.get(tick) {
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
                            ), // Source rectangle (entire texture)
                            note_rect,              // Destination rectangle
                            Vector2::new(0.0, 0.0), // Origin (top-left corner)
                            0.0,                    // Rotation
                            Color::WHITE,           // Tint color
                        );

                        // Draw the tone (note name) on the note
                        let text = &piano_key.label; // Use the note label (e.g., "C4", "D#5")
                        let text_width = d.measure_text(text, 10); // Measure text width
                        let text_x = note_rect.x + (note_rect.width - text_width as f32) / 2.0; // Center text horizontally
                        let text_y = note_rect.y + (note_rect.height - 10.0) / 2.0; // Center text vertically

                        d.draw_text(
                            text,
                            text_x as i32,
                            text_y as i32,
                            10,
                            Color::WHITE, // Text color
                        );
                    }
                }
            }
        }

        d.clear_background(Color::DARKGRAY);
        d.draw_text(&title, 12, 12, 20, Color::BLACK);

        piano::draw_piano_keys(window_width, window_height, &all_keys, &piano_props, d);
    }
}
