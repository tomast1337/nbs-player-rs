extern crate raylib;

use raylib::prelude::*;
use utils::time_formatter;

mod audio;
mod font;
mod note;
mod piano;
mod song;
mod textures;
mod theme;
mod utils;

fn main() {
    let mut window_width = 1280.;
    let mut window_height = 720.;

    let nbs_data = song::load_nbs_file(None);

    let nbs_file = nbs_data.song;
    let extra_sounds = nbs_data.extra_sounds;

    if extra_sounds.len() == 0 {
        log::warn!("No extra sounds loaded");
    } else {
        log::warn!("{:?}", nbs_file.instruments);
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
    rl.set_trace_log_callback(utils::logger_callback).unwrap();
    rl.set_target_fps(60);

    let textures = textures::load_textures(&mut rl, &thread);
    let theme = theme::Theme::default();

    let (mut all_keys, key_map) = piano::generate_piano_keys();

    let mut piano_props;
    let mut note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);
    log::debug!("Loaded note blocks");
    log::debug!("Loaded {} notes", note_blocks.len());

    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(Some(extra_sounds), 0.5);

    log::debug!("Loaded audio engine");
    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time: f32 = 0.; // Elapsed time in seconds

    let mut note_dim;
    let mut key_spacing; // Spacing between keys

    let mut played_ticks: Vec<bool> = vec![false; nbs_file.header.song_length as usize];

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused: bool = true;

    let font = font::load_fonts(0, &mut rl, &thread);
    window_width = rl.get_screen_width() as f32;
    window_height = rl.get_screen_height() as f32;
    piano_props = piano::initialize_piano_dimensions(window_width, &all_keys, &font);
    note_dim = piano_props.white_key_width;
    key_spacing = piano_props.key_spacing;

    let mut volume = 0.5; // Volume level (0.0 to 1.0)
    audio_engine.set_global_volume(volume);

    let controls_close_time = 5.0; // Time in seconds to wait before closing controls
    let mut sec_since_last_mouse_move = 0.0; // Timer for mouse inactivity
    let mut last_mouse_pos = rl.get_mouse_position(); // Last recorded mouse position
    let mut controls_panel_y = window_height; // Initial position of the controls panel (hidden)
    let control_panel_height = 80.0; // Height of the control panel
    let button_size = Vector2::new(40.0, 40.0); // Size of play/pause and reset buttons
    let volume_button_size = Vector2::new(30.0, 30.0); // Size of volume buttons
    let timeline_height = 10.0; // Height of the timeline slider
    let mut toggle_fullscreen = false; // Fullscreen state

    while !rl.window_should_close() {
        if toggle_fullscreen {
            rl.toggle_fullscreen();
            toggle_fullscreen = false;
        }

        update_window_dimensions(
            &mut window_width,
            &mut window_height,
            &mut rl,
            &all_keys,
            &mut piano_props,
            &mut note_dim,
            &mut key_spacing,
            &font,
        );

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

        // Check if the mouse has moved
        let current_mouse_pos = rl.get_mouse_position();
        if current_mouse_pos.x != last_mouse_pos.x || current_mouse_pos.y != last_mouse_pos.y {
            sec_since_last_mouse_move = 0.0; // Reset the inactivity timer
            last_mouse_pos = current_mouse_pos; // Update the last mouse position
        } else {
            sec_since_last_mouse_move += delta_time; // Increment the inactivity timer
        }

        current_tick = elapsed_time * notes_per_second;

        // Reset all key press states
        reset_key_press_states(&mut all_keys);

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

        if sec_since_last_mouse_move < controls_close_time {
            // Slide the panel up (show)
            controls_panel_y =
                utils::lerp(controls_panel_y, window_height - control_panel_height, 0.2);
        } else {
            // Slide the panel down (hide)
            controls_panel_y = utils::lerp(controls_panel_y, window_height, 0.2);
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background_color);

        // Draw notes
        let notes_rendered = note::draw_notes(
            &mut d,
            window_width,
            window_height,
            &all_keys,
            &key_map,
            &note_blocks,
            &piano_props,
            &textures.note_texture,
            current_tick,
            note_dim,
            key_spacing,
            &instrument_colors,
            &font,
        );
        // Update and draw piano keys
        piano::update_key_animation(&mut all_keys, delta_time);
        piano::draw_piano_keys(
            &mut d,
            window_width,
            window_height,
            &all_keys,
            &piano_props,
            &textures.piano_key_texture,
            &theme,
        );

        // Calculate font size based on screen width with min and max limits
        let font_size = draw_song_status(
            &mut d,
            &theme,
            window_width,
            &title,
            total_duration,
            current_tick,
            elapsed_time,
            &font,
            notes_rendered,
        );

        // Draw FPS in the top-right corner
        d.draw_fps(window_width as i32 - 100, 10);

        let is_end = elapsed_time >= total_duration;

        // Draw pause state
        if is_paused && !is_end {
            draw_pause_message(window_width, window_height, &theme, &textures, &mut d);
        }

        if is_end {
            draw_end_message(
                window_width,
                window_height,
                &theme,
                &title,
                &font,
                &mut d,
                font_size,
            );
        }

        {
            let control_panel_rect =
                Rectangle::new(0.0, controls_panel_y, window_width, control_panel_height);
            d.draw_rectangle_rec(control_panel_rect, Color::BLACK.alpha(0.7));

            // Draw the play/pause button
            let play_pause_button_rect = Rectangle::new(
                button_size.x,
                controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
                button_size.x,
                button_size.y,
            );
            d.draw_texture_pro(
                if is_paused {
                    &textures.play_button
                } else {
                    &textures.pause_button
                },
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.play_button.width as f32,
                    textures.play_button.height as f32,
                ),
                Rectangle::new(
                    play_pause_button_rect.x,
                    play_pause_button_rect.y,
                    play_pause_button_rect.width,
                    play_pause_button_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            // Draw the reset button
            let reset_button_rect = Rectangle::new(
                button_size.x * 2.0 + 10.0,
                controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
                button_size.x,
                button_size.y,
            );
            d.draw_texture_pro(
                &textures.reset_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.reset_button.width as f32,
                    textures.reset_button.height as f32,
                ),
                Rectangle::new(
                    reset_button_rect.x,
                    reset_button_rect.y,
                    reset_button_rect.width,
                    reset_button_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            let space_from_buttons = button_size.x * 3.0 + 10.0;

            // Draw the timeline slider
            let timeline_rect = Rectangle::new(
                space_from_buttons + 10.0,
                controls_panel_y + control_panel_height / 2.0 - timeline_height / 2.0,
                window_width - 2. * space_from_buttons - 20.0,
                timeline_height,
            );
            d.draw_rectangle_rec(timeline_rect, Color::BLACK);

            // Draw the current progress on the timeline
            let progress_width = (elapsed_time / total_duration) * timeline_rect.width;
            let progress_rect = Rectangle::new(
                timeline_rect.x,
                timeline_rect.y,
                progress_width,
                timeline_rect.height,
            );
            d.draw_rectangle_rec(progress_rect, theme.accent_color);

            // draw the timeline pill
            let timeline_pill_rect = Rectangle::new(
                timeline_rect.x + progress_width - 5.0,
                timeline_rect.y - 5.0,
                timeline_rect.height + 10.0,
                timeline_rect.height + 10.0,
            );
            d.draw_texture_pro(
                &textures.timeline_pill,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.timeline_pill.width as f32,
                    textures.timeline_pill.height as f32,
                ),
                Rectangle::new(
                    timeline_pill_rect.x,
                    timeline_pill_rect.y,
                    timeline_pill_rect.width,
                    timeline_pill_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            // Draw the volume controls
            let volume_plus_rect = Rectangle::new(
                space_from_buttons + 20.0 + timeline_rect.width,
                controls_panel_y + control_panel_height / 2.0 - volume_button_size.y / 2.0,
                volume_button_size.x,
                volume_button_size.y,
            );
            d.draw_texture_pro(
                &textures.volume_plus_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.volume_plus_button.width as f32,
                    textures.volume_plus_button.height as f32,
                ),
                Rectangle::new(
                    volume_plus_rect.x,
                    volume_plus_rect.y,
                    volume_plus_rect.width,
                    volume_plus_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            let volume_minus_rect = Rectangle::new(
                volume_plus_rect.x + volume_plus_rect.width + 10.0,
                volume_plus_rect.y,
                volume_button_size.x,
                volume_button_size.y,
            );
            d.draw_texture_pro(
                &textures.volume_minus_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.volume_minus_button.width as f32,
                    textures.volume_minus_button.height as f32,
                ),
                Rectangle::new(
                    volume_minus_rect.x,
                    volume_minus_rect.y,
                    volume_minus_rect.width,
                    volume_minus_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            // fullscreen button
            let fullscreen_button_rect = Rectangle::new(
                volume_minus_rect.x + volume_minus_rect.width + 10.0,
                volume_minus_rect.y,
                volume_button_size.x,
                volume_button_size.y,
            );
            d.draw_texture_pro(
                &textures.fullscreen_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.fullscreen_button.width as f32,
                    textures.fullscreen_button.height as f32,
                ),
                Rectangle::new(
                    fullscreen_button_rect.x,
                    fullscreen_button_rect.y,
                    fullscreen_button_rect.width,
                    fullscreen_button_rect.height,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );
            // Check for button clicks
            if d.is_mouse_button_pressed(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
                let mouse_pos = d.get_mouse_position();

                // Check if the play/pause button was clicked
                if play_pause_button_rect.check_collision_point_rec(mouse_pos) {
                    is_paused = !is_paused;
                }

                // Check if the reset button was clicked
                if reset_button_rect.check_collision_point_rec(mouse_pos) {
                    elapsed_time = 0.;
                    played_ticks = vec![false; nbs_file.header.song_length as usize];
                    note_blocks = note::get_note_blocks(&nbs_file);
                    is_paused = false;
                }

                // Check if the volume plus button was clicked
                if volume_plus_rect.check_collision_point_rec(mouse_pos) {
                    volume += 0.1;
                    if volume > 1.0 {
                        volume = 1.0;
                    }
                    audio_engine.set_global_volume(volume);
                }

                // Check if the volume minus button was clicked
                if volume_minus_rect.check_collision_point_rec(mouse_pos) {
                    volume -= 0.1;
                    if volume < 0.0 {
                        volume = 0.0;
                    }
                    audio_engine.set_global_volume(volume);
                }

                // Check if the timeline was clicked
                if timeline_rect.check_collision_point_rec(mouse_pos) {
                    let new_x = mouse_pos.x - timeline_rect.x;
                    let new_progress = new_x / timeline_rect.width;
                    elapsed_time = new_progress * total_duration;
                    current_tick = elapsed_time * notes_per_second;
                    // set all played ticks to false beyond the current tick and all ticks before as played
                    for i in 0..(current_tick as f32).floor() as usize {
                        played_ticks[i] = true;
                    }
                    for i in (current_tick as f32).floor() as usize..played_ticks.len() {
                        played_ticks[i] = false;
                    }
                }

                // Check if the fullscreen button was clicked
                if fullscreen_button_rect.check_collision_point_rec(mouse_pos) {
                    toggle_fullscreen = !toggle_fullscreen;
                }
            }

            // check left mouse button pressed
            if d.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
                // Check if the timeline was clicked
                if timeline_rect.check_collision_point_rec(current_mouse_pos) {
                    let new_x = current_mouse_pos.x - timeline_rect.x;
                    let new_progress = new_x / timeline_rect.width;
                    elapsed_time = new_progress * total_duration;
                    current_tick = elapsed_time * notes_per_second;
                    // set all played ticks to false beyond the current tick and all ticks before as played
                    for i in 0..(current_tick as f32).floor() as usize {
                        played_ticks[i] = true;
                    }
                    for i in (current_tick as f32).floor() as usize..played_ticks.len() {
                        played_ticks[i] = false;
                    }
                }
            }

            // Draw the current time and total duration
            let current_time_text = format!(
                "{} / {}",
                time_formatter(elapsed_time),
                time_formatter(total_duration)
            );
            d.draw_text_pro(
                &font,
                &current_time_text,
                Vector2::new(timeline_rect.x, timeline_rect.y - 25.),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                theme.text_color,
            );
        }
    }
}

fn draw_pause_message(
    window_width: f32,
    window_height: f32,
    theme: &theme::Theme,
    textures: &textures::Textures,
    d: &mut RaylibDrawHandle<'_>,
) {
    d.draw_texture_pro(
        &textures.play_button,
        Rectangle::new(
            0.0,
            0.0,
            textures.play_button.width as f32,
            textures.play_button.height as f32,
        ),
        Rectangle::new(
            window_width / 2. - 50.,
            window_height / 2. - 50.,
            100.,
            100.,
        ),
        Vector2::new(0.0, 0.0),
        0.0,
        theme.accent_color,
    );
}

fn draw_end_message(
    window_width: f32,
    window_height: f32,
    theme: &theme::Theme,
    title: &String,
    font: &Font,
    d: &mut RaylibDrawHandle<'_>,
    font_size: f32,
) {
    d.draw_text_pro(
        font,
        "End of Song",
        Vector2::new(window_width / 2. - 50., window_height / 2.),
        Vector2::new(0.0, 0.0),
        0.0,
        font_size,
        0.,
        theme.accent_color,
    );
    d.draw_text_pro(
        font,
        title,
        Vector2::new(window_width / 2. - 50., window_height / 2. + 50.),
        Vector2::new(0.0, 0.0),
        0.0,
        font_size,
        0.,
        theme.accent_color,
    );
    d.draw_text_pro(
        font,
        "Press Space to Restart",
        Vector2::new(window_width / 2. - 50., window_height / 2. + 100.),
        Vector2::new(0.0, 0.0),
        0.0,
        font_size,
        0.,
        theme.accent_color,
    );
}

fn draw_song_status(
    d: &mut RaylibDrawHandle<'_>,
    theme: &theme::Theme,
    window_width: f32,
    title: &String,
    total_duration: f32,
    current_tick: f32,
    elapsed_time: f32,
    font: &Font,
    notes_rendered: i32,
) -> f32 {
    let min_font_size = 18.;
    let max_font_size = 40.;
    let font_size = (window_width / 64.0).clamp(min_font_size, max_font_size as f32);

    // Define text positions
    let start_x = 10.0;
    let mut start_y = 10.0;
    let line_height = font.measure_text(title, font_size, 0.0).y;

    // Define text color
    let text_color = theme.text_color;

    // Draw song status

    let current_tick_text = format!("Current Tick: {:.4}", current_tick);
    let notes_rendered_text = format!("Notes Rendered: {}", notes_rendered);
    let duration_text = format!(
        "Duration: {}|{}",
        time_formatter(elapsed_time),
        time_formatter(total_duration)
    );
    d.draw_text_pro(
        font,
        title,
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
        font,
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
        font,
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
        font,
        &current_tick_text,
        Vector2::new(start_x, start_y),
        Vector2::new(0.0, 0.0),
        0.0,
        font_size,
        0.,
        text_color,
    );
    font_size
}

fn reset_key_press_states(all_keys: &mut Vec<piano::PianoKey>) {
    for key in all_keys {
        key.is_pressed = false;
    }
}

fn update_window_dimensions<'a>(
    window_width: &mut f32,
    window_height: &mut f32,
    rl: &mut RaylibHandle,
    all_keys: &Vec<piano::PianoKey>,
    piano_props: &mut piano::PianoProps<'a>,
    note_dim: &mut f32,
    key_spacing: &mut f32,
    font: &'a Font,
) {
    if *window_width as i32 != rl.get_screen_height() {
        *window_width = rl.get_screen_width() as f32;
        *piano_props = piano::initialize_piano_dimensions(*window_width, all_keys, font);
        *note_dim = piano_props.white_key_width;
        *key_spacing = piano_props.key_spacing;
    }
    if *window_height as i32 != rl.get_screen_height() {
        *window_height = rl.get_screen_height() as f32;
    }
}
