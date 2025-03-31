extern crate raylib;
use raylib::prelude::GuiControl::*;
use raylib::prelude::GuiControlProperty::*;
use raylib::prelude::*;
use simple_logger::SimpleLogger;
use std::env;
use utils::time_formatter;

mod audio;
mod config;
mod font;
mod note;
mod piano;
mod song;
mod textures;
mod theme;
mod utils;

fn main() {
    // Initialize the logger
    match SimpleLogger::new().init() {
        Ok(_) => log::info!("Logger initialized"),
        Err(err) => {
            eprintln!("Failed to initialize logger: {}", err);
            std::process::exit(1);
        }
    };

    let data = match utils::load_file("song.nbsx") {
        Ok(data) => data,
        Err(err) => {
            log::error!("Error loading file: {}", err);
            std::process::exit(1);
        }
    };

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        log::error!("Usage: {} <json-config>", args[0]);
        std::process::exit(1);
    }

    let json_arg = &args[1];

    // Try to parse the JSON argument
    let config: config::AppConfig = match serde_json::from_str(json_arg) {
        Ok(parsed) => parsed,
        Err(err) => {
            log::error!("Error parsing JSON: {}", err);
            std::process::exit(1);
        }
    };

    // Example validation: Check if width and height are within a reasonable range
    if config.window_width <= 0 || config.window_height <= 0 {
        log::error!("Error: Window dimensions must be greater than 0");
        std::process::exit(1);
    }
    if config.window_width < 200 || config.window_height < 200 {
        log::error!("Error: Window dimensions are too small");
        std::process::exit(1);
    }

    log::info!("Parsed config: {:?}", config);

    let mut window_width = config.window_width as f32;
    let mut window_height = config.window_height as f32;

    let nbs_data = song::load_nbs_file(Some(&data));

    let nbs_file = nbs_data.song;
    let extra_sounds = nbs_data.extra_sounds;

    let song_name: String = match String::from_utf8(nbs_file.header.song_name.clone()) {
        Ok(name) => name,
        Err(_err) => "Error converting song name".to_string(),
    };
    let song_author: String = match String::from_utf8(nbs_file.header.song_author.clone()) {
        Ok(author) => author,
        Err(_err) => "Error converting song author".to_string(),
    };
    let title: String = format!("{} - {}", song_name, song_author);

    let notes_per_second: f32 = nbs_file.header.tempo as f32 / 100.0;
    let total_duration: f32 = nbs_file.header.song_length as f32 / notes_per_second;

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title(&title)
        .build();
    rl.set_target_fps(60);

    rl.gui_enable();

    let textures = textures::load_textures(&mut rl, &thread);
    let theme = theme::Theme::from_theme_config(&config.theme);
    let font = font::load_fonts(config.font_id as usize, &mut rl, &thread);

    rl.gui_set_font(&font);

    let (mut all_keys, key_map) = piano::generate_piano_keys();
    let mut piano_props;
    let mut note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);
    log::debug!("Loaded note blocks");
    log::debug!("Loaded {} notes", note_blocks.len());

    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time: f32 = 0.; // Elapsed time in seconds

    let mut note_dim;
    let mut key_spacing; // Spacing between keys

    let mut played_ticks: Vec<bool> = vec![false; nbs_file.header.song_length as usize];

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused: bool = true;

    window_width = rl.get_screen_width() as f32;
    window_height = rl.get_screen_height() as f32;
    piano_props = piano::initialize_piano_dimensions(window_width, &all_keys, &font);
    note_dim = piano_props.white_key_width;
    key_spacing = piano_props.key_spacing;

    let mut volume = 0.5; // Volume level (0.0 to 1.0)
    // audio_engine.set_global_volume(volume);
    let raylib_audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio device");
    let mut audio_engine: audio::AudioEngine =
        audio::AudioEngine::new(&raylib_audio, Some(extra_sounds));

    let controls_close_time = 0.5; // Time in seconds to wait before closing controls
    let mut sec_since_last_mouse_move = 0.0; // Timer for mouse inactivity
    let mut last_mouse_pos = rl.get_mouse_position(); // Last recorded mouse position
    let mut controls_panel_y = window_height; // Initial position of the controls panel (hidden)
    let button_size = Vector2::new(30.0, 30.0); // Size of volume buttons
    let control_panel_height = button_size.y * 2.0; // Height of the control panel
    let timeline_height = button_size.y; // Height of the timeline slider
    let mut toggle_fullscreen = false; // Fullscreen state

    set_gui_style(&mut rl, &theme);

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

        // get current tick notes to play if not paused
        if !is_paused
            && elapsed_time < total_duration
            && !played_ticks[(current_tick as f32).floor() as usize]
        {
            // Play the notes for the current tick
            if let Some(notes) = note_blocks.get(current_tick as usize) {
                audio_engine.play_tick(notes);
                //sound.play();
                played_ticks[(current_tick as f32).floor() as usize] = true;
            }
        }

        // Trigger piano key presses for current and trigger audio
        if let Some(notes) = note_blocks.get_mut(current_tick as usize) {
            for note in notes {
                if let Some(&key_index) = key_map.get(&note.key) {
                    all_keys[key_index].is_pressed = true;
                }
            }
        }

        if sec_since_last_mouse_move < controls_close_time {
            // Slide the panel up (show)
            controls_panel_y =
                utils::lerp(controls_panel_y, window_height - control_panel_height, 0.2);
            //rl.hide_cursor();
        } else {
            // Slide the panel down (hide)
            controls_panel_y = utils::lerp(controls_panel_y, window_height, 0.2);
            //rl.show_cursor();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background_color);

        // Draw notes
        note::draw_notes(
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

        let min_font_size = 18.;
        let max_font_size = 40.;
        let font_size = (window_width / 64.0).clamp(min_font_size, max_font_size as f32);

        // Calculate font size based on screen width with min and max limits
        draw_song_status(&mut d, &theme, &title, &font, font_size);

        // Draw FPS in the top-right corner
        d.draw_fps(window_width as i32 - 100, 10);

        let is_end = elapsed_time >= total_duration;

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

        unsafe {
            let control_panel_rect =
                Rectangle::new(0.0, controls_panel_y, window_width, control_panel_height);
            d.draw_rectangle_rec(control_panel_rect, Color::BLACK.alpha(0.7));

            // Draw the play/pause button
            let play_pause_button_rect = Rectangle::new(
                button_size.x / 2.,
                controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
                button_size.x,
                button_size.y,
            );

            let is_play_pause_click = ffi::GuiButton(
                play_pause_button_rect.into(),
                utils::string_to_c_char("".to_string()),
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
                play_pause_button_rect,
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            // Draw the reset button
            let reset_button_rect = Rectangle::new(
                button_size.x + button_size.x / 2. + 10.0,
                controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
                button_size.x,
                button_size.y,
            );

            let is_reset_click = ffi::GuiButton(
                reset_button_rect.into(),
                utils::string_to_c_char("".to_string()),
            );

            d.draw_texture_pro(
                &textures.reset_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.reset_button.width as f32,
                    textures.reset_button.height as f32,
                ),
                reset_button_rect,
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            // Draw the timeline slider
            let timeline_rect = Rectangle::new(
                reset_button_rect.x + reset_button_rect.width + 10.0,
                controls_panel_y + control_panel_height / 2.0 - timeline_height / 2.0,
                window_width - reset_button_rect.width - button_size.x * 5.5 - 30.0,
                timeline_height,
            );

            let mut new_tick = current_tick;
            let is_timeline_slider_adjusted = ffi::GuiSlider(
                timeline_rect.into(),
                utils::string_to_c_char("".to_string()),
                utils::string_to_c_char("".to_string()),
                &mut new_tick,
                0.0,
                nbs_file.header.song_length as f32,
            );

            let current_time_text = format!(
                "{} / {}",
                time_formatter(elapsed_time),
                time_formatter(total_duration)
            );

            // Measure the size of the text
            let text_size = font.measure_text(&current_time_text, font_size, 0.);

            // Calculate the position to center the text on the timeline
            let text_x = timeline_rect.x + 10.;
            let text_y = timeline_rect.y + (timeline_rect.height / 2.0) - (text_size.y / 2.0);

            // Draw the text centered on the timeline
            d.draw_text_pro(
                &font,
                &current_time_text,
                Vector2::new(text_x, text_y),
                Vector2::new(0.0, 0.0),
                0.0,
                font_size,
                0.,
                theme.accent_color,
            );

            // Draw the volume controls
            let volume_rect = Rectangle::new(
                timeline_rect.x + timeline_rect.width + 10.0,
                controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
                button_size.x * 2.0,
                button_size.y,
            );

            let is_volume_adjusted = ffi::GuiSliderBar(
                volume_rect.into(),
                utils::string_to_c_char("".to_string()),
                utils::string_to_c_char("".to_string()),
                &mut volume,
                0.0,
                1.0,
            );

            let volume_texture = if volume == 0.0 {
                &textures.vol_000
            } else if volume <= 0.25 {
                &textures.vol_025
            } else if volume <= 0.5 {
                &textures.vol_050
            } else if volume <= 0.75 {
                &textures.vol_075
            } else {
                &textures.vol_100
            };
            d.draw_texture_pro(
                volume_texture,
                Rectangle::new(
                    0.0,
                    0.0,
                    volume_texture.width as f32,
                    volume_texture.height as f32,
                ),
                Rectangle::new(
                    volume_rect.x + button_size.x / 2.,
                    volume_rect.y,
                    button_size.x,
                    button_size.y,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color.alpha(0.5),
            );

            // Draw the fullscreen button
            let fullscreen_button_rect = Rectangle::new(
                volume_rect.x + volume_rect.width + 10.0,
                volume_rect.y,
                button_size.x,
                button_size.y,
            );

            let is_fullscreen_click = ffi::GuiButton(
                fullscreen_button_rect.into(),
                utils::string_to_c_char("".to_string()),
            );

            if is_paused {
                // daw a button on the middle of the screen to unpause
                let foo = Rectangle::new(
                    window_width / 2. - 50.,
                    window_height / 2. - 50.,
                    100.,
                    100.,
                );

                let is_button_clicked =
                    ffi::GuiButton(foo.into(), utils::string_to_c_char("".to_string()));

                d.draw_texture_pro(
                    &textures.play_button,
                    Rectangle::new(
                        0.0,
                        0.0,
                        textures.play_button.width as f32,
                        textures.play_button.height as f32,
                    ),
                    foo,
                    Vector2::new(0.0, 0.0),
                    0.0,
                    theme.accent_color,
                );

                if is_button_clicked == 1 {
                    is_paused = false;
                }
            }

            d.draw_texture_pro(
                &textures.fullscreen_button,
                Rectangle::new(
                    0.0,
                    0.0,
                    textures.fullscreen_button.width as f32,
                    textures.fullscreen_button.height as f32,
                ),
                fullscreen_button_rect,
                Vector2::new(0.0, 0.0),
                0.0,
                theme.accent_color,
            );

            if is_play_pause_click == 1 {
                is_paused = !is_paused;
            }

            if is_reset_click == 1 {
                elapsed_time = 0.;
                played_ticks = vec![false; nbs_file.header.song_length as usize];
                is_paused = true;
            }

            if is_timeline_slider_adjusted == 1 {
                // set all played ticks to false beyond the current tick and all ticks before as played
                current_tick = new_tick;
                for i in 0..nbs_file.header.song_length as usize {
                    if i < current_tick as usize {
                        played_ticks[i] = true;
                    } else {
                        played_ticks[i] = false;
                    }
                }
                elapsed_time = new_tick / notes_per_second;
            }

            if is_volume_adjusted == 1 {
                raylib_audio.set_master_volume(volume);
            }

            if is_fullscreen_click == 1 {
                toggle_fullscreen = !toggle_fullscreen;
            }

            // while the cursor is in the control panel, do not move the panel down
            if control_panel_rect
                .check_collision_point_rec(Vector2::new(last_mouse_pos.x, last_mouse_pos.y))
            {
                sec_since_last_mouse_move = 0.0;
            }
        }
    }
}

fn set_gui_style(rl: &mut RaylibHandle, theme: &theme::Theme) {
    // ------------------------------BUTTON STYLE------------------------------
    rl.gui_set_style(
        BUTTON,
        BASE_COLOR_NORMAL,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        BASE_COLOR_FOCUSED,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        BASE_COLOR_PRESSED,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        TEXT_COLOR_NORMAL,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        BORDER_COLOR_NORMAL,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        BORDER_COLOR_PRESSED,
        theme.accent_color.color_to_int(),
    );
    rl.gui_set_style(
        BUTTON,
        BORDER_COLOR_FOCUSED,
        theme.accent_color.color_to_int(),
    );
    // ------------------------------SLIDER STYLE------------------------------
    rl.gui_set_style(
        SLIDER,
        BASE_COLOR_NORMAL,
        theme.background_color.color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        BASE_COLOR_FOCUSED,
        theme.black_key_color.color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        BASE_COLOR_PRESSED,
        theme.white_key_color.brightness(0.9).color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        BORDER_COLOR_NORMAL,
        Color::WHITE.alpha(0.).color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        BORDER_COLOR_PRESSED,
        theme.accent_color.color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        BORDER_COLOR_FOCUSED,
        theme.accent_color.color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        TEXT_COLOR_NORMAL,
        theme.white_key_color.alpha(1.).color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        TEXT_COLOR_FOCUSED,
        theme.accent_color.color_to_int(),
    );
    rl.gui_set_style(
        SLIDER,
        TEXT_COLOR_PRESSED,
        theme.accent_color.color_to_int(),
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
    let measure = font.measure_text(title, font_size, 0.0);
    d.draw_text_pro(
        font,
        title,
        Vector2::new(window_width / 2. - measure.x / 2., window_height / 2. - 50.),
        Vector2::new(0.0, 0.0),
        0.0,
        font_size,
        0.,
        theme.accent_color,
    );
    let measure = font
        .measure_text("Press Space to Restart", font_size, 0.0)
        .x;
    d.draw_text_pro(
        font,
        "Press Space to Restart",
        Vector2::new(window_width / 2. - measure / 2., window_height / 2. + 10.),
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
    title: &String,
    font: &Font,
    font_size: f32,
) -> f32 {
    // Define text positions
    let start_x = 10.0;
    let start_y = 10.0;

    // Define text color
    let text_color = theme.text_color;

    // Draw song status
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
    if *window_width as i32 != rl.get_screen_width() {
        *window_width = rl.get_screen_width() as f32;
        *piano_props = piano::initialize_piano_dimensions(*window_width, all_keys, font);
        *note_dim = piano_props.white_key_width;
        *key_spacing = piano_props.key_spacing;
    }
    if *window_height as i32 != rl.get_screen_height() {
        *window_height = rl.get_screen_height() as f32;
    }
}
