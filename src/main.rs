use log;
use raylib::prelude::*;
use song::load_nbs_file;

mod audio;
mod note;
mod piano;
mod song;

fn logger_callback(level: TraceLogLevel, text: &str) {
    match level {
        TraceLogLevel::LOG_ALL => log::trace!("{}", text),
        TraceLogLevel::LOG_TRACE => log::trace!("{}", text),
        TraceLogLevel::LOG_DEBUG => log::debug!("{}", text),
        TraceLogLevel::LOG_INFO => log::info!("{}", text),
        TraceLogLevel::LOG_WARNING => log::warn!("{}", text),
        TraceLogLevel::LOG_ERROR => log::error!("{}", text),
        TraceLogLevel::LOG_FATAL => log::error!("{}", text),
        TraceLogLevel::LOG_NONE => {}
    }
}

fn time_formatter(time: f32) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0) as u32;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

fn main() {
    colog::init();
    let window_width: f32 = 1280.;
    let window_height = 720.;

    let nbs_file = load_nbs_file(None);

    let song_name = String::from_utf8(nbs_file.header.song_name.clone()).unwrap();
    let song_author = String::from_utf8(nbs_file.header.song_author.clone()).unwrap();
    let title = format!("{} - {}", song_name, song_author);
    let notes_per_second = nbs_file.header.tempo as f32 / 100.0;
    let total_duration = nbs_file.header.song_length as f32 / notes_per_second;

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title(title.as_str())
        .build();

    rl.set_trace_log_callback(logger_callback).unwrap();

    let (mut all_keys, key_map) = piano::generate_piano_keys();
    let mut note_blocks = note::get_note_blocks(&nbs_file);

    let piano_props = piano::initialize_piano_dimensions(
        window_width,
        window_height,
        &all_keys,
        &mut rl,
        &thread,
    );

    let note_texture = note::load_note_texture(&mut rl, &thread);

    let mut audio_engine = audio::AudioEngine::new(None, 0.5);

    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time = 0.0; // Elapsed time in seconds

    let note_dim = piano_props.white_key_width;
    let key_spacing = piano_props.key_spacing; // Spacing between keys

    let mut played_ticks = vec![false; nbs_file.header.song_length as usize];

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);
        elapsed_time += delta_time;

        // Show FPS counter
        d.draw_fps(window_width as i32 - 100, 10);
        // Clear background and draw UI
        d.clear_background(Color::SKYBLUE);
        // Update current tick based on elapsed time and tempo
        current_tick = elapsed_time * notes_per_second;

        // Reset all key press states
        for key in &mut all_keys {
            key.is_pressed = false;
        }

        // get current tick notes to play
        if let Some(notes) = note_blocks.get((current_tick as f32).floor() as usize) {
            // if tick notes are not played, play them
            if !played_ticks[(current_tick as f32).floor() as usize] {
                audio_engine.play_tick(notes);
                //notes.iter().for_each(|note| audio_engine._play_sound(note));

                played_ticks[(current_tick as f32).floor() as usize] = true;
            }
        }

        // Trigger piano key presses for current and trigger audio
        if let Some(notes) = note_blocks.get_mut(current_tick as usize) {
            for note in notes {
                if note.was_played == false {
                    if let Some(&key_index) = key_map.get(&note.key) {
                        all_keys[key_index].is_pressed = true;
                    }
                    note.was_played = true;
                }
            }
        }

        // Draw notes
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

                        let y_pos =
                            window_height - ((tick_f32 - current_tick) * note_dim) - note_dim;

                        // Check if the note is visible on the screen
                        if y_pos + note_dim > min_y && y_pos < max_y {
                            let note_rect = Rectangle::new(
                                x_pos + window_width / 2.0 - note_dim / 2.0,
                                y_pos,
                                note_dim,
                                note_dim,
                            );

                            d.draw_texture_pro(
                                &note_texture,
                                texture_source_rect,
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

                            // Increment notes rendered count
                            notes_rendered += 1;
                        }
                    }
                }
            }
        }

        // Update and draw piano keys
        piano::update_key_animation(&mut all_keys, delta_time);
        piano::draw_piano_keys(window_width, window_height, &all_keys, &piano_props, &mut d);

        // Draw song status
        let duration = format!(
            "Duration: {}|{}",
            time_formatter(elapsed_time),
            time_formatter(total_duration)
        );
        let notes_redered = format!("Notes Rendered: {}", notes_rendered);
        let current_tick = format!("Current Tick: {:.4}", current_tick);

        d.draw_text(&title, 12, 12, 20, Color::BLACK);
        d.draw_text(&duration, 12, 42, 20, Color::BLACK);
        d.draw_text(&notes_redered, 12, 72, 20, Color::BLACK);
        d.draw_text(&current_tick, 12, 102, 20, Color::BLACK);
    }
}
