use std::collections::HashMap;

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

    if nbs_file.instruments.len() == 0 {
        log::warn!("No extra sounds loaded");
    }

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

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused = true;

    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
            is_paused = !is_paused;
        }

        // Update elapsed time if not paused ad song is not finished
        if !is_paused && elapsed_time < total_duration {
            elapsed_time += delta_time;
        }

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
        let notes_rendered = note::draw_notes(
            window_width,
            window_height,
            &all_keys,
            &key_map,
            &note_blocks,
            &piano_props,
            &note_texture,
            current_tick,
            note_dim,
            key_spacing,
            &mut d,
            &instrument_colors,
        );

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

        d.draw_text(&title, 2, 2, 10, Color::BLACK);
        d.draw_text(&duration, 2, 12, 10, Color::BLACK);
        d.draw_text(&notes_redered, 2, 24, 10, Color::BLACK);
        d.draw_text(&current_tick, 2, 36, 10, Color::BLACK);

        // Draw pause state
        if is_paused {
            d.draw_text(
                "PAUSED",
                window_width as i32 / 2 - 40,
                window_height as i32 / 2 - 20,
                40,
                Color::RED,
            );
        }
    }
}
