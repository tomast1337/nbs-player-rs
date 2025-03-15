use macroquad::{
    self, color,
    input::{KeyCode, is_key_pressed},
    text::draw_text,
    time::{get_fps, get_frame_time},
    window::{self, clear_background, request_new_screen_size},
};

mod audio;
mod note;
mod piano;
mod song;

fn time_formatter(time: f32) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0) as u32;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

#[macroquad::main("BasicShapes")]
async fn main() {
    colog::init();
    let mut window_width = 1280.;
    let mut window_height = 720.;

    request_new_screen_size(window_width, window_height);

    let nbs_file: nbs_rs::NbsFile = song::load_nbs_file(None);

    if nbs_file.instruments.len() == 0 {
        log::warn!("No extra sounds loaded");
    }

    if nbs_file.instruments.len() == 0 {
        log::warn!("No extra sounds loaded");
    }

    let song_name: String = String::from_utf8(nbs_file.header.song_name.clone()).unwrap();
    let song_author: String = String::from_utf8(nbs_file.header.song_author.clone()).unwrap();
    let title: String = format!("{} - {}", song_name, song_author);
    let notes_per_second: f32 = nbs_file.header.tempo as f32 / 100.0;
    let total_duration: f32 = nbs_file.header.song_length as f32 / notes_per_second;

    let (mut all_keys, key_map) = piano::generate_piano_keys();

    let mut piano_props;
    let mut note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);

    let note_texture = note::load_note_texture();

    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(None, 0.5);

    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time: f32 = 0.; // Elapsed time in seconds

    let mut note_dim;
    let mut key_spacing; // Spacing between keys

    let mut played_ticks: Vec<bool> = vec![false; nbs_file.header.song_length as usize];

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused: bool = true;

    loop {
        window_width = window::screen_width();
        window_height = window::screen_height();
        piano_props = piano::initialize_piano_dimensions(window_width, window_height, &all_keys);
        note_dim = piano_props.white_key_width;
        key_spacing = piano_props.key_spacing;

        let delta_time = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            is_paused = !is_paused;
        }

        // Update elapsed time if not paused ad song is not finished
        if !is_paused && elapsed_time < total_duration {
            elapsed_time += delta_time;
        }
        clear_background(color::SKYBLUE);

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
            &instrument_colors,
        );

        // Update and draw piano keys
        piano::update_key_animation(&mut all_keys, delta_time);
        piano::draw_piano_keys(window_width, window_height, &all_keys, &piano_props);

        // Calculate font size based on screen width
        let font_size = window_width / 64.0;

        // Draw song status
        let fps = get_fps();
        let current_tick_text = format!("Current Tick: {:.4}", current_tick);
        let notes_rendered_text = format!("Notes Rendered: {}", notes_rendered);
        let duration_text = format!(
            "Duration: {}|{}",
            time_formatter(total_duration), //time_formatter(elapsed_time),
            time_formatter(total_duration)
        );

        draw_text(&title, 10., 15., font_size, color::BLACK);
        draw_text(&duration_text, 10., 35., font_size, color::BLACK);
        draw_text(&notes_rendered_text, 10., 55., font_size, color::BLACK);

        draw_text(&current_tick_text, 10., 75., font_size, color::BLACK);
        draw_text(
            &format!("FPS: {:.2}", fps),
            window_width - 100.,
            font_size,
            20.,
            color::BLACK,
        );
        // Draw pause state
        if is_paused {
            draw_text(
                "Paused",
                window_width / 2. - 50.,
                window_height / 2.,
                font_size * 2.5,
                color::RED,
            );
        }

        window::next_frame().await
    }
}
