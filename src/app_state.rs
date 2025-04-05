extern crate raylib;
use std::collections::HashMap;

use nbs_rs::NbsFile;
use raylib::prelude::*;

use crate::audio;
use crate::audio::AudioClip;
use crate::config;
use crate::font;
use crate::note;
use crate::piano;
use crate::song;
use crate::textures;
use crate::theme;
use crate::utils;

#[derive(Debug, Clone)]
pub struct PianoState {
    pub all_keys: Vec<piano::PianoKey>,
    pub key_map: std::collections::HashMap<u8, usize>,
    pub piano_props: piano::PianoProps,
}
impl PianoState {
    fn new(window_width: f32, font: &Font) -> Self {
        let (all_keys, key_map) = piano::generate_piano_keys();
        let piano_props = piano::initialize_piano_dimensions(window_width, &all_keys, font);
        PianoState {
            all_keys,
            key_map,
            piano_props,
        }
    }
    fn reset_keys(&mut self) {
        for key in &mut self.all_keys {
            key.is_pressed = false;
        }
    }
    fn trigger_key_press(
        &mut self,
        note_blocks: &mut Vec<note::NoteBlock>,
        note_color_map: &HashMap<u32, Color>,
    ) {
        for note in note_blocks {
            let instrument = note.instrument;
            if let Some(&key_index) = self.key_map.get(&note.key) {
                let color = note_color_map.get(&(instrument)).unwrap_or(&Color::WHITE);
                let tint = (color, note.volume / 5.);
                self.all_keys[key_index].press(Some(tint));
            }
        }
    }
    fn update_key_animation(&mut self, delta_time: f32) {
        piano::update_key_animation(&mut self.all_keys, delta_time);
    }
}
#[derive(Debug, Clone)]
pub struct SongState<'a> {
    pub note_blocks: Vec<Vec<note::NoteBlock>>, // Note blocks for each tick
    pub current_tick: f32,                      // Current tick of the song
    pub elapsed_time: f32,                      // Elapsed time since the song started
    pub note_dim: f32,                          // Dimension of the note blocks
    pub font_size_3: f32,                       // Font size for the note blocks with sharp notes
    pub font_size_2: f32,                       // Font size for the note blocks with flat notes
    pub key_spacing: f32,                       // Spacing between keys
    pub played_ticks: Vec<bool>,                // Vector to track played ticks
    pub instrument_colors: HashMap<u32, Color>, // Map of instrument colors
    pub is_paused: bool,                        // Flag to check if the song is paused
    pub nbs_file: &'a NbsFile,                  // Reference to the NBS file
    pub extra_sounds: Vec<(&'a [u8], f64)>,     // Extra sounds to be played
    pub title: String,                          // Title of the song
    pub notes_per_second: f32,                  // Notes per second based on the tempo
    pub total_duration: f32,                    // Total duration of the song
    pub is_end: bool,                           // Flag to check if the song has ended
}

impl SongState<'_> {
    fn draw_song_status(&self, d: &mut RaylibDrawHandle<'_>, app_state: &AppState) -> f32 {
        // Define text positions
        let start_x = 10.0;
        let start_y = 10.0;

        // Define text color
        let text_color = app_state.theme.text_color;
        let font = &app_state.font;
        let font_size = app_state.font_size;
        let title = &self.title;

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
}

#[derive(Debug, Clone)]
pub struct ControlsState {
    pub controls_close_time: f32, // Time in seconds to wait before closing controls
    pub sec_since_last_mouse_move: f32, // Timer for mouse inactivity
    pub last_mouse_pos: Vector2,  // Last recorded mouse position
    pub controls_panel_y: f32,    // Initial position of the controls panel (hidden)
    pub button_size: Vector2,     // Size of volume buttons
    pub control_panel_height: f32, // Height of the control panel
    pub timeline_height: f32,     // Height of the timeline slider
    pub toggle_fullscreen: bool,  // Fullscreen state
}

impl ControlsState {
    fn new(window_height: f32) -> Self {
        const BTN_S: f32 = 40.; // Size of the buttons
        let button_size = Vector2::new(BTN_S, BTN_S);
        let control_panel_height = button_size.y * 2.0;
        ControlsState {
            controls_close_time: 0.5, // Time in seconds to wait before closing controls
            sec_since_last_mouse_move: 0.0, // Timer for mouse inactivity
            last_mouse_pos: Vector2::zero(), // Last recorded mouse position
            button_size: button_size.clone(), // Size of buttons
            control_panel_height,     // Height of the control panel
            timeline_height: button_size.y, // Height of the timeline slider
            toggle_fullscreen: false, // Fullscreen call state
            controls_panel_y: window_height - control_panel_height, // Initial position of the controls panel (hidden)
        }
    }
    fn update_mouse_state(&mut self, current_mouse_pos: Vector2, delta_time: f32) {
        if current_mouse_pos.x != self.last_mouse_pos.x
            || current_mouse_pos.y != self.last_mouse_pos.y
        {
            self.sec_since_last_mouse_move = 0.0; // Reset the inactivity timer
            self.last_mouse_pos = current_mouse_pos; // Update the last mouse position
        } else {
            self.sec_since_last_mouse_move += delta_time; // Increment the inactivity timer
        }
    }

    fn update_controls_panel_position(&mut self, window_height: f32) {
        if self.sec_since_last_mouse_move < self.controls_close_time {
            // Slide the panel up (show)
            self.controls_panel_y = utils::lerp(
                self.controls_panel_y,
                window_height - self.control_panel_height,
                0.2,
            );
        } else {
            // Slide the panel down (hide)
            self.controls_panel_y = utils::lerp(self.controls_panel_y, window_height, 0.2);
        }
    }
}

#[derive(Debug)]
pub struct AppState<'a> {
    pub window_width: f32,
    pub window_height: f32,
    pub textures: textures::Textures,
    pub theme: theme::Theme,
    pub font: Font,
    pub font_size: f32,
    pub volume: f32,
    pub song_state: SongState<'a>,
    pub piano_state: PianoState,
    pub controls_state: ControlsState,
}

impl<'a> AppState<'a> {
    pub fn setup_application(
        config: config::AppConfig,
        nbs_data: &'a song::SongData<'a>,
    ) -> (AppState<'a>, RaylibHandle, RaylibThread) {
        let window_width = config.window_width as f32;
        let window_height = config.window_height as f32;

        // Check if width and height are within a reasonable range
        if config.window_width <= 0 || config.window_height <= 0 {
            log::error!("Error: Window dimensions must be greater than 0");
            std::process::exit(1);
        }
        if config.window_width < 200 || config.window_height < 200 {
            log::error!("Error: Window dimensions are too small");
            std::process::exit(1);
        }

        let nbs_file = &nbs_data.song;
        let extra_sounds = &nbs_data.extra_sounds;

        let song_name: String = match String::from_utf8(nbs_file.header.song_name.clone()) {
            Ok(name) => name,
            Err(_err) => "Unknown".to_string(),
        };
        let song_author: String = match String::from_utf8(nbs_file.header.song_author.clone()) {
            Ok(author) => author,
            Err(_err) => "Unknown".to_string(),
        };
        let title: String = format!("{} - {}", song_name, song_author);

        let notes_per_second: f32 = nbs_file.header.tempo as f32 / 100.;
        let total_duration: f32 = nbs_file.header.song_length as f32 / notes_per_second;

        /* ------------------------------Raylib------------------------------ */

        let (mut rl, thread) = raylib::init()
            .size(window_width as i32, window_height as i32)
            .title(&title)
            .build();

        let textures = textures::load_textures(&mut rl, &thread);
        let theme = theme::Theme::from_theme_config(&config.theme);
        let font = font::load_fonts(config.font_id as usize, &mut rl, &thread);

        rl.set_target_fps(config.target_fps.unwrap_or(60));
        rl.gui_enable();

        /* ----------------------------Piano State--------------------------- */
        let piano_state = PianoState::new(window_width, &font);

        /* ----------------------------Note State---------------------------- */
        let note_dim = piano_state.piano_props.white_key_width;
        let (font_size_3, font_size_2) = note::calculate_note_block_font_sizes(note_dim, &font);
        let song_state = SongState {
            note_blocks: Vec::new(),
            current_tick: 0.,
            elapsed_time: 0.,
            note_dim,
            font_size_3,
            font_size_2,
            key_spacing: piano_state.piano_props.key_spacing,
            played_ticks: vec![false; nbs_file.header.song_length as usize],
            instrument_colors: note::generate_instrument_palette(),
            is_paused: true,
            nbs_file,
            extra_sounds: extra_sounds.to_vec(),
            title,
            notes_per_second,
            total_duration,
            is_end: false,
        };
        /* --------------------------Controls State-------------------------- */
        let controls_state = ControlsState::new(window_height);

        let app_state = AppState {
            window_width,
            window_height,
            textures,
            theme,
            font,
            font_size: 0.0,
            volume: config.initial_volume.unwrap_or(0.5),
            song_state,
            piano_state,
            controls_state,
        };

        rl.gui_set_font(&app_state.font); // Set the font for the GUI
        app_state.theme.set_gui_style(&mut rl); // Apply the theme to the GUI
        return (app_state, rl, thread);
    }

    pub fn update_window_dimensions(&mut self, rl: &mut RaylibHandle) {
        let new_width = rl.get_screen_width() as f32;
        let new_height = rl.get_screen_height() as f32;
        let window_width = self.window_width;
        let window_height = self.window_height;
        let song_state = &mut self.song_state;
        let font = &self.font;

        if window_width != new_width {
            self.window_width = new_width;
            self.piano_state.piano_props = piano::initialize_piano_dimensions(
                self.window_width,
                &self.piano_state.all_keys,
                &self.font,
            );
            song_state.note_dim = self.piano_state.piano_props.white_key_width;
            song_state.key_spacing = self.piano_state.piano_props.key_spacing;

            let min_font_size = 18.;
            let max_font_size = 40.;
            self.font_size = (self.window_width / 64.0).clamp(min_font_size, max_font_size as f32);

            let (font_size_3, font_size_2) =
                note::calculate_note_block_font_sizes(song_state.note_dim, &font);

            song_state.font_size_3 = font_size_3;
            song_state.font_size_2 = font_size_2;
        }
        if window_height != new_height {
            self.window_height = new_height;
        }
    }

    pub fn toggle_fullscreen(&mut self, rl: &mut RaylibHandle) {
        if self.controls_state.toggle_fullscreen {
            rl.toggle_fullscreen();
            self.controls_state.toggle_fullscreen = false;
        }
    }

    pub fn update(
        &mut self,
        rl: &mut RaylibHandle,
        delta_time: f32,
        sounds: &HashMap<u32, AudioClip>,
    ) {
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) {
            if self.song_state.elapsed_time >= self.song_state.total_duration {
                self.song_state.elapsed_time = 0.;
                self.song_state.played_ticks =
                    vec![false; self.song_state.nbs_file.header.song_length as usize];
                self.song_state.note_blocks =
                    note::get_note_blocks(&self.song_state.nbs_file, &sounds);
                self.song_state.is_paused = false;
            }
            self.song_state.is_paused = !self.song_state.is_paused;
        }

        // Update elapsed time if not paused ad song is not finished
        if !self.song_state.is_paused
            && self.song_state.elapsed_time < self.song_state.total_duration
        {
            self.song_state.elapsed_time += delta_time;
        }

        // Check if the mouse has moved
        let current_mouse_pos = rl.get_mouse_position();
        self.controls_state
            .update_mouse_state(current_mouse_pos, delta_time);

        self.song_state.current_tick =
            self.song_state.elapsed_time * self.song_state.notes_per_second;

        // Reset all key press states
        self.piano_state.reset_keys();

        // Trigger piano key presses for current and trigger audio
        if let Some(notes) = self
            .song_state
            .note_blocks
            .get_mut(self.song_state.current_tick as usize)
        {
            self.piano_state
                .trigger_key_press(notes, &self.song_state.instrument_colors);
        }

        // Update the controls panel position based on mouse activity
        self.controls_state
            .update_controls_panel_position(self.window_height);

        self.piano_state.update_key_animation(delta_time); // Update key animations

        // is end of song
        self.song_state.is_end = self.song_state.elapsed_time >= self.song_state.total_duration;
    }

    pub fn update_audio(&mut self, audio_engine: &mut audio::AudioEngine) {
        // Skip if paused, finished, or already played this tick
        if self.song_state.is_paused
            || self.song_state.elapsed_time >= self.song_state.total_duration
            || self.song_state.played_ticks[self.song_state.current_tick as usize]
        {
            return;
        }

        // Check if there are notes to play for the current tick
        if let Some(notes) = self
            .song_state
            .note_blocks
            .get(self.song_state.current_tick as usize)
        {
            audio_engine.play_tick(notes);

            // Mark this tick as played
            self.song_state.played_ticks[self.song_state.current_tick as usize] = true;
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        d.clear_background(self.theme.background_color);
        // Draw notes
        note::draw_notes(d, self);
        // draw piano keys
        piano::draw_piano_keys(d, self);
        // daw song status
        self.song_state.draw_song_status(d, self);

        self.draw_end_message(d);
        // Draw FPS in the top-right corner
        d.draw_fps(self.window_width as i32 - 100, 10);
    }

    fn draw_end_message(&self, d: &mut RaylibDrawHandle<'_>) {
        if !self.song_state.is_end {
            return;
        }
        let window_width = self.window_width;
        let window_height = self.window_height;
        let theme = &self.theme;
        let font = &self.font;
        let font_size = self.font_size;
        let title = &self.song_state.title;
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

    pub fn update_and_draw_gui(
        &mut self,
        d: &mut RaylibDrawHandle<'_>,
        raylib_audio: &RaylibAudio,
    ) {
        let controls_panel_y = self.controls_state.controls_panel_y;
        let control_panel_height = self.controls_state.control_panel_height;
        let window_width = self.window_width;
        let window_height = self.window_height;
        let font = &self.font;
        let font_size = self.font_size;
        let button_size = self.controls_state.button_size;
        let timeline_height = self.controls_state.timeline_height;
        let total_duration = self.song_state.total_duration;
        let elapsed_time = self.song_state.elapsed_time;
        let last_mouse_pos = self.controls_state.last_mouse_pos;
        let control_panel_rect =
            Rectangle::new(0.0, controls_panel_y, window_width, control_panel_height);
        /* ------------------------ GUI elements Rect ----------------------- */
        let play_pause_button_rect = Rectangle::new(
            button_size.x / 2.,
            controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
            button_size.x,
            button_size.y,
        );
        let reset_button_rect = Rectangle::new(
            button_size.x + button_size.x / 2. + 10.0,
            controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
            button_size.x,
            button_size.y,
        );

        let timeline_rect = Rectangle::new(
            reset_button_rect.x + reset_button_rect.width + 10.0,
            controls_panel_y + control_panel_height / 2.0 - timeline_height / 2.0,
            window_width - reset_button_rect.width - button_size.x * 5.5 - 30.0,
            timeline_height,
        );

        let current_time_text = format!(
            "{} / {}",
            utils::time_formatter(elapsed_time),
            utils::time_formatter(total_duration)
        );

        // Measure the size of the text
        let time_line_text_size = font.measure_text(&current_time_text, font_size, 0.);

        // Calculate the position to center the text on the timeline
        let text_x = timeline_rect.x + 10.;
        let text_y = timeline_rect.y + (timeline_rect.height / 2.0) - (time_line_text_size.y / 2.0);

        let volume_rect = Rectangle::new(
            timeline_rect.x + timeline_rect.width + 10.0,
            controls_panel_y + control_panel_height / 2.0 - button_size.y / 2.0,
            button_size.x * 2.0,
            button_size.y,
        );

        let fullscreen_button_rect = Rectangle::new(
            volume_rect.x + volume_rect.width + 10.0,
            volume_rect.y,
            button_size.x,
            button_size.y,
        );

        // while the cursor is in the control panel, do not move the panel down
        if control_panel_rect
            .check_collision_point_rec(Vector2::new(last_mouse_pos.x, last_mouse_pos.y))
        {
            self.controls_state.sec_since_last_mouse_move = 0.0;
        }
        /*----------------------------- Draw Gui -----------------------------*/
        let is_paused = self.song_state.is_paused;
        let textures = &self.textures;
        let theme = &self.theme;
        let nbs_file = &self.song_state.nbs_file;
        let notes_per_second = self.song_state.notes_per_second;
        let font = &self.font;
        let font_size = self.font_size;
        let current_tick = self.song_state.current_tick;
        unsafe {
            d.draw_rectangle_rec(control_panel_rect, Color::BLACK.alpha(0.7));

            // Draw the play/pause button

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
            let mut new_tick = current_tick;
            let is_timeline_slider_adjusted = ffi::GuiSlider(
                timeline_rect.into(),
                utils::string_to_c_char("".to_string()),
                utils::string_to_c_char("".to_string()),
                &mut new_tick,
                0.0,
                nbs_file.header.song_length as f32,
            );

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
            let is_volume_adjusted = ffi::GuiSliderBar(
                volume_rect.into(),
                utils::string_to_c_char("".to_string()),
                utils::string_to_c_char("".to_string()),
                &mut self.volume,
                0.0,
                1.0,
            );

            let volume = self.volume;
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
                    self.song_state.is_paused = false;
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
                self.song_state.is_paused = !self.song_state.is_paused;
            }

            if is_reset_click == 1 {
                self.song_state.elapsed_time = 0.;
                self.song_state.played_ticks = vec![false; nbs_file.header.song_length as usize];
                self.song_state.is_paused = true;
            }

            if is_timeline_slider_adjusted == 1 {
                // set all played ticks to false beyond the current tick and all ticks before as played
                self.song_state.current_tick = new_tick;
                for i in 0..nbs_file.header.song_length as usize {
                    if i < current_tick as usize {
                        self.song_state.played_ticks[i] = true;
                    } else {
                        self.song_state.played_ticks[i] = false;
                    }
                }
                self.song_state.elapsed_time = new_tick / notes_per_second;
            }

            if is_volume_adjusted == 1 {
                raylib_audio.set_master_volume(volume);
            }

            if is_fullscreen_click == 1 {
                self.controls_state.toggle_fullscreen = !self.controls_state.toggle_fullscreen;
            }
        }
    }
}
