extern crate raylib;
use nbs_player_core::audio::AudioBackend;
use nbs_player_core::config::AppConfig;
use nbs_player_core::controls::ControlsState;
use nbs_player_core::notes::{self, NoteBlock};
use nbs_player_core::piano::PianoState;
use nbs_player_core::player::Player;
use nbs_player_core::profiler;
use nbs_player_core::theme::Theme;
use nbs_player_core::utils;
use raylib::prelude::*;

use crate::background::Background;
use crate::font;
use crate::render::{self, RlText, color, from_vec2, rect, tex_src};
use crate::textures;
use crate::theme as gui_theme;

#[derive(Debug)]
pub struct AppState {
    pub window_width: f32,
    pub window_height: f32,
    pub textures: textures::Textures,
    pub theme: Theme,
    pub font: Font,
    pub font_size: f32,
    pub volume: f32,
    pub player: Player,
    pub piano_state: PianoState,
    pub controls_state: ControlsState,
    // Note block geometry, derived from the piano layout.
    pub note_dim: f32,
    pub key_spacing: f32,
    pub font_size_3: f32,
    pub font_size_2: f32,
    background_shader: Background,
}

/// UI font size for a given window width.
fn ui_font_size(window_width: f32) -> f32 {
    (window_width / 64.0).clamp(18., 40.)
}

impl AppState {
    pub fn setup_application(
        config: AppConfig,
        song: &nbs_rs::NbsFile,
        note_blocks: Vec<Vec<NoteBlock>>,
    ) -> (AppState, RaylibHandle, RaylibThread) {
        let window_width = config.window_width as f32;
        let window_height = config.window_height as f32;

        if config.window_width == 0 || config.window_height == 0 {
            log::error!("Error: Window dimensions must be greater than 0");
            std::process::exit(1);
        }
        if config.window_width < 200 || config.window_height < 200 {
            log::error!("Error: Window dimensions are too small");
            std::process::exit(1);
        }

        let song_name = String::from_utf8(song.header.song_name.clone())
            .unwrap_or_else(|_| "Unknown".to_string());
        let song_author = String::from_utf8(song.header.song_author.clone())
            .unwrap_or_else(|_| "Unknown".to_string());
        let title = format!("{} - {}", song_name, song_author);

        let player = Player::new(
            title.clone(),
            song.header.tempo,
            song.header.song_length as usize,
            note_blocks,
            notes::generate_instrument_palette(),
        );

        /* ------------------------------Raylib------------------------------ */
        let (mut rl, thread) = raylib::init()
            .size(window_width as i32, window_height as i32)
            .title(&title)
            .build();

        let textures = textures::load_textures(&mut rl, &thread);
        let theme = Theme::from_theme_config(&config.theme);
        let font = font::load_fonts(config.font_id, &mut rl, &thread);

        rl.set_target_fps(config.target_fps.unwrap_or(60));
        rl.gui_enable();

        let background_shader = Background::new(&mut rl, &thread, config.background);

        /* ------------------------------Layout------------------------------ */
        let piano_state = PianoState::new(window_width, &RlText(&font));
        let note_dim = piano_state.piano_props.white_key_width;
        let (font_size_3, font_size_2) =
            notes::calculate_note_block_font_sizes(note_dim, &RlText(&font));

        let app_state = AppState {
            background_shader,
            window_width,
            window_height,
            textures,
            theme,
            font,
            font_size: ui_font_size(window_width),
            volume: config.initial_volume.unwrap_or(0.5),
            player,
            key_spacing: piano_state.piano_props.key_spacing,
            piano_state,
            controls_state: ControlsState::new(window_height),
            note_dim,
            font_size_3,
            font_size_2,
        };

        rl.gui_set_font(&app_state.font);
        gui_theme::set_gui_style(&app_state.theme, &mut rl);
        (app_state, rl, thread)
    }

    pub fn update_window_dimensions(&mut self, rl: &mut RaylibHandle) {
        let new_width = rl.get_screen_width() as f32;
        let new_height = rl.get_screen_height() as f32;

        if self.window_width != new_width {
            self.window_width = new_width;
            let measure = RlText(&self.font);
            self.piano_state.resize(new_width, &measure);
            self.note_dim = self.piano_state.piano_props.white_key_width;
            self.key_spacing = self.piano_state.piano_props.key_spacing;
            self.font_size = ui_font_size(new_width);

            let (font_size_3, font_size_2) =
                notes::calculate_note_block_font_sizes(self.note_dim, &measure);
            self.font_size_3 = font_size_3;
            self.font_size_2 = font_size_2;
        }
        self.window_height = new_height;
    }

    pub fn toggle_fullscreen(&mut self, rl: &mut RaylibHandle) {
        if self.controls_state.toggle_fullscreen {
            rl.toggle_fullscreen();
            self.controls_state.toggle_fullscreen = false;
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, delta_time: f32) {
        if !self.player.is_paused {
            self.background_shader.shader_time += delta_time;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            self.player.space_pressed();
        }
        self.player.update(delta_time);

        let mouse = from_vec2(rl.get_mouse_position());
        self.controls_state.update_mouse_state(mouse, delta_time);

        // Reset all key press states, then press the keys under the playhead.
        self.piano_state.reset_keys();
        if let Some(notes) = self.player.notes_at_playhead() {
            self.piano_state
                .trigger_key_press(notes, &self.player.instrument_colors);
        }

        self.controls_state
            .update_controls_panel_position(self.window_height);
        self.piano_state.update_key_animation(delta_time);
    }

    pub fn update_audio(&mut self, audio: &mut dyn AudioBackend) {
        let _p = profiler::scope("update_audio");
        if let Some(notes) = self.player.take_due_notes() {
            audio.play_tick(notes);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle<'_>) {
        let _p = profiler::scope("draw");
        {
            let _p = profiler::scope("background");
            self.background_shader
                .draw(d, &self.theme, [self.window_width, self.window_height]);
        }
        {
            let _p = profiler::scope("notes");
            render::draw_notes(d, self);
        }
        {
            let _p = profiler::scope("piano");
            render::draw_piano_keys(d, self);
        }
        {
            let _p = profiler::scope("status");
            self.draw_song_status(d);
        }

        self.draw_end_message(d);
        d.draw_fps(self.window_width as i32 - 100, 10);
    }

    fn draw_song_status(&self, d: &mut RaylibDrawHandle<'_>) {
        d.draw_text_pro(
            &self.font,
            &self.player.title,
            Vector2::new(10.0, 10.0),
            Vector2::zero(),
            0.0,
            self.font_size,
            0.,
            color(self.theme.text_color),
        );
    }

    fn draw_end_message(&self, d: &mut RaylibDrawHandle<'_>) {
        if !self.player.is_end {
            return;
        }
        let (w, h) = (self.window_width, self.window_height);
        let accent = color(self.theme.accent_color);
        let mut line = |text: &str, dy: f32| {
            let measure = self.font.measure_text(text, self.font_size, 0.0).x;
            d.draw_text_pro(
                &self.font,
                text,
                Vector2::new(w / 2. - measure / 2., h / 2. + dy),
                Vector2::zero(),
                0.0,
                self.font_size,
                0.,
                accent,
            );
        };
        line(&self.player.title, -50.);
        line("Press Space to Restart", 10.);
    }

    /// Draws the control panel with raygui and applies what the user clicked.
    pub fn update_and_draw_gui(
        &mut self,
        d: &mut RaylibDrawHandle<'_>,
        audio: &mut dyn AudioBackend,
    ) {
        let layout = self
            .controls_state
            .layout(self.window_width, self.window_height);
        self.controls_state.hold_open_if_hovered(layout.panel);

        let textures = &self.textures;
        let accent = color(self.theme.accent_color);
        let is_paused = self.player.is_paused;
        let button_size = self.controls_state.button_size;

        let time_text = format!(
            "{} / {}",
            utils::time_formatter(self.player.elapsed_time),
            utils::time_formatter(self.player.total_duration)
        );
        let time_size = self.font.measure_text(&time_text, self.font_size, 0.);
        let time_pos = Vector2::new(
            layout.timeline.x + 10.,
            layout.timeline.y + layout.timeline.h / 2.0 - time_size.y / 2.0,
        );

        let draw_icon = |d: &mut RaylibDrawHandle<'_>, tex: &Texture2D, dst: Rectangle, tint: Color| {
            d.draw_texture_pro(tex, tex_src(tex), dst, Vector2::zero(), 0.0, tint);
        };

        let play_pause_clicked;
        let reset_clicked;
        let mut center_play_clicked = false;
        let fullscreen_clicked;
        let mut seek_to: Option<f32> = None;
        let volume_changed;

        // raygui draws and polls in one call; the empty label must not allocate per frame.
        let empty = c"".as_ptr();
        unsafe {
            d.draw_rectangle_rec(rect(layout.panel), Color::BLACK.alpha(0.7));

            // Play/pause
            play_pause_clicked = ffi::GuiButton(rect(layout.play_pause).into(), empty) == 1;
            draw_icon(
                d,
                if is_paused {
                    &textures.play_button
                } else {
                    &textures.pause_button
                },
                rect(layout.play_pause),
                accent,
            );

            // Reset
            reset_clicked = ffi::GuiButton(rect(layout.reset).into(), empty) == 1;
            draw_icon(d, &textures.reset_button, rect(layout.reset), accent);

            // Timeline
            let mut new_tick = self.player.current_tick;
            if ffi::GuiSlider(
                rect(layout.timeline).into(),
                empty,
                empty,
                &mut new_tick,
                0.0,
                self.player.song_length as f32,
            ) == 1
            {
                seek_to = Some(new_tick);
            }
            d.draw_text_pro(
                &self.font,
                &time_text,
                time_pos,
                Vector2::zero(),
                0.0,
                self.font_size,
                0.,
                accent,
            );

            // Volume
            volume_changed = ffi::GuiSliderBar(
                rect(layout.volume).into(),
                empty,
                empty,
                &mut self.volume,
                0.0,
                1.0,
            ) == 1;
            let volume_texture = match self.volume {
                v if v == 0.0 => &textures.vol_000,
                v if v <= 0.25 => &textures.vol_025,
                v if v <= 0.5 => &textures.vol_050,
                v if v <= 0.75 => &textures.vol_075,
                _ => &textures.vol_100,
            };
            draw_icon(
                d,
                volume_texture,
                Rectangle::new(
                    layout.volume.x + button_size.x / 2.,
                    layout.volume.y,
                    button_size.x,
                    button_size.y,
                ),
                accent.alpha(0.5),
            );

            // Fullscreen
            fullscreen_clicked = ffi::GuiButton(rect(layout.fullscreen).into(), empty) == 1;

            // Big play button while paused
            if is_paused {
                center_play_clicked = ffi::GuiButton(rect(layout.center_play).into(), empty) == 1;
                draw_icon(d, &textures.play_button, rect(layout.center_play), accent);
            }

            draw_icon(d, &textures.fullscreen_button, rect(layout.fullscreen), accent);
        }

        if center_play_clicked {
            self.player.play();
        }
        if play_pause_clicked {
            self.player.toggle_pause();
        }
        if reset_clicked {
            self.player.reset();
        }
        if let Some(tick) = seek_to {
            self.player.seek_to_tick(tick);
        }
        if volume_changed {
            audio.set_master_volume(self.volume);
        }
        if fullscreen_clicked {
            self.controls_state.toggle_fullscreen = !self.controls_state.toggle_fullscreen;
        }
    }
}
