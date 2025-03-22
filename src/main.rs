extern crate raylib;

use raylib::prelude::*;
use utils::time_formatter;

mod audio;
mod font;
mod note;
mod piano;
mod song;
mod utils;

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

fn main() {
    let mut window_width = 1280;
    let mut window_height = 720;

    let nbs_data = song::load_nbs_file(None);

    let nbs_file = nbs_data.song;
    let extra_sounds = nbs_data.extra_sounds;

    if extra_sounds.len() == 0 {
        log::warn!("No extra sounds loaded");
    } else {
        println!("{:?}", nbs_file.instruments);
    }

    let song_name: String = String::from_utf8(nbs_file.header.song_name.clone()).unwrap();
    let song_author: String = String::from_utf8(nbs_file.header.song_author.clone()).unwrap();
    let title: String = format!("{} - {}", song_name, song_author);
    let notes_per_second: f32 = nbs_file.header.tempo as f32 / 100.0;
    let total_duration: f32 = nbs_file.header.song_length as f32 / notes_per_second;

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title(&title)
        .resizable()
        .build();
    rl.set_trace_log_callback(logger_callback).unwrap();
    rl.set_target_fps(60);
    let (mut all_keys, key_map) = piano::generate_piano_keys();

    let mut piano_props;
    let mut note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);
    println!("Loaded note blocks");
    println!("Loaded {} notes", note_blocks.len());
    let note_texture = note::load_note_texture(&mut rl, &thread);

    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(Some(extra_sounds), 0.5);

    println!("Loaded audio engine");
    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time: f32 = 0.; // Elapsed time in seconds

    let mut note_dim;
    let mut key_spacing; // Spacing between keys

    let mut played_ticks: Vec<bool> = vec![false; nbs_file.header.song_length as usize];

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused: bool = true;

    let font = font::load_fonts(4, &mut rl, &thread);
    window_width = rl.get_screen_width();
    window_height = rl.get_screen_height();
    piano_props =
        piano::initialize_piano_dimensions(window_width as f32, &all_keys, &mut rl, &thread);
    note_dim = piano_props.white_key_width;
    key_spacing = piano_props.key_spacing;
    while !rl.window_should_close() {
        if window_width != rl.get_screen_height() {
            window_width = rl.get_screen_width();
            piano_props = piano::initialize_piano_dimensions(
                window_width as f32,
                &all_keys,
                &mut rl,
                &thread,
            );
            note_dim = piano_props.white_key_width;
            key_spacing = piano_props.key_spacing;
        }
        if window_height != rl.get_screen_height() {
            window_height = rl.get_screen_height();
        }

        let delta_time = rl.get_frame_time();

        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) {
            if elapsed_time >= total_duration {
                elapsed_time = 0.;
                played_ticks = vec![false; nbs_file.header.song_length as usize];
                note_blocks = note::get_note_blocks(&nbs_file);
                is_paused = false;
            }
            is_paused = !is_paused;
        }

        // Update elapsed time if not paused ad song is not finished
        if !is_paused && elapsed_time < total_duration {
            elapsed_time += delta_time;
        }
        //clear_background(color::SKYBLUE);

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
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::SKYBLUE);

        // Draw notes
        let notes_rendered = note::draw_notes(
            &mut d,
            window_width as f32,
            window_height as f32,
            &all_keys,
            &key_map,
            &note_blocks,
            &piano_props,
            &note_texture,
            current_tick,
            note_dim,
            key_spacing,
            &instrument_colors,
        );

        // Update and draw piano keys
        piano::update_key_animation(&mut all_keys, delta_time);
        piano::draw_piano_keys(
            &mut d,
            window_width as f32,
            window_height as f32,
            &all_keys,
            &piano_props,
            &font,
        );

        // Calculate font size based on screen width with min and max limits
        let min_font_size = 18.;
        let max_font_size = 40.;
        let font_size = (window_width as f32 / 64.0).clamp(min_font_size, max_font_size as f32);

        // Define text positions
        let start_x = 10.0;
        let mut start_y = 10.0;
        let line_height = font.measure_text(&title, font_size, 0.0).y;

        // Define text color
        let text_color = Color::BLACK;

        // Draw song status

        let current_tick_text = format!("Current Tick: {:.4}", current_tick);
        let notes_rendered_text = format!("Notes Rendered: {}", notes_rendered);
        let duration_text = format!(
            "Duration: {}|{}",
            time_formatter(elapsed_time),
            time_formatter(total_duration)
        );
        d.draw_text_pro(
            &font,
            &title,
            Vector2::new(start_x, start_y),
            Vector2::new(0.0, 0.0),
            0.0,
            font_size as f32,
            0.,
            text_color,
        );
        // Draw duration
        start_y += line_height;
        d.draw_text_pro(
            &font,
            &duration_text,
            Vector2::new(start_x, start_y),
            Vector2::new(0.0, 0.0),
            0.0,
            font_size,
            0.,
            text_color,
        );

        // Draw notes rendered
        start_y += line_height;
        d.draw_text_pro(
            &font,
            &notes_rendered_text,
            Vector2::new(start_x, start_y),
            Vector2::new(0.0, 0.0),
            0.0,
            font_size,
            0.,
            text_color,
        );

        start_y += line_height;
        d.draw_text_pro(
            &font,
            &current_tick_text,
            Vector2::new(start_x, start_y),
            Vector2::new(0.0, 0.0),
            0.0,
            font_size,
            0.,
            text_color,
        );

        // Draw FPS in the top-right corner
        d.draw_fps(window_width as i32 - 100, 10);

        let is_end = elapsed_time >= total_duration;

        // Draw pause state
        if is_paused && !is_end {
            d.draw_text_pro(
                &font,
                "Paused",
                Vector2::new(window_width as f32 / 2. - 50., window_height as f32 / 2.),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                Color::RED,
            );
        }

        if is_end {
            d.draw_text_pro(
                &font,
                "End of Song",
                Vector2::new(window_width as f32 / 2. - 50., window_height as f32 / 2.),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                Color::RED,
            );
            d.draw_text_pro(
                &font,
                &title,
                Vector2::new(
                    window_width as f32 / 2. - 50.,
                    window_height as f32 / 2. + 50.,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                Color::BLACK,
            );
            d.draw_text_pro(
                &font,
                "Press Space to Restart",
                Vector2::new(
                    window_width as f32 / 2. - 50.,
                    window_height as f32 / 2. + 100.,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                Color::BLACK,
            );
        }
    }
}
