//! The whole player as a backend-independent state machine: feed it input, time, a
//! [`Renderer`] and an [`AudioBackend`] once per frame.

use crate::audio::AudioBackend;
use crate::controls::ControlsState;
use crate::debug_overlay;
use crate::notes::{self, NoteScene};
use crate::piano::{self, PianoState};
use crate::player::Player;
use crate::profiler;
use crate::render::{InputState, Renderer, Sprite};
use crate::theme::Theme;
use crate::types::{Rect, Rgba, TextMeasure, Vec2};
use crate::ui::{SliderStyle, Ui};
use crate::utils;

const ID_PLAY_PAUSE: u32 = 1;
const ID_RESET: u32 = 2;
const ID_TIMELINE: u32 = 3;
const ID_VOLUME: u32 = 4;
const ID_FULLSCREEN: u32 = 5;
const ID_CENTER_PLAY: u32 = 6;

/// Things the frontend has to do because only it can (window management).
#[derive(Debug, Default, Clone, Copy)]
pub struct AppEvents {
    pub toggle_fullscreen: bool,
}

pub struct App {
    pub theme: Theme,
    pub player: Player,
    pub piano_state: PianoState,
    pub controls_state: ControlsState,
    pub volume: f32,
    pub window_width: f32,
    pub window_height: f32,
    ui: Ui,
    font_size: f32,
    // Note block geometry, derived from the piano layout.
    note_dim: f32,
    key_spacing: f32,
    font_size_3: f32,
    font_size_2: f32,
    /// Drives the animated background; only advances while playing.
    background_time: f32,
    /// Smoothed frames per second for the corner counter.
    fps: f32,
}

/// UI font size for a given window width.
fn ui_font_size(window_width: f32) -> f32 {
    (window_width / 64.0).clamp(18., 40.)
}

impl App {
    pub fn new(
        theme: Theme,
        player: Player,
        window_size: Vec2,
        initial_volume: f32,
        background_time: f32,
        measure: &dyn TextMeasure,
    ) -> Self {
        let piano_state = PianoState::new(window_size.x, measure);
        let note_dim = piano_state.piano_props.white_key_width;
        let (font_size_3, font_size_2) = notes::calculate_note_block_font_sizes(note_dim, measure);
        App {
            theme,
            player,
            key_spacing: piano_state.piano_props.key_spacing,
            piano_state,
            controls_state: ControlsState::new(window_size.y),
            volume: initial_volume,
            window_width: window_size.x,
            window_height: window_size.y,
            ui: Ui::default(),
            font_size: ui_font_size(window_size.x),
            note_dim,
            font_size_3,
            font_size_2,
            background_time,
            fps: 0.0,
        }
    }

    /// Run one frame: update state, release due notes to `audio`, draw everything.
    pub fn frame(
        &mut self,
        input: &InputState,
        dt: f32,
        window_size: Vec2,
        r: &mut dyn Renderer,
        audio: &mut dyn AudioBackend,
    ) -> AppEvents {
        profiler::end_frame();
        if input.toggle_profiler {
            profiler::toggle();
        }
        if input.dump_profile {
            profiler::dump_folded();
        }

        {
            let _p = profiler::scope("update");
            self.resize(window_size, r);
            self.update(input, dt);
        }
        self.update_audio(audio);

        if dt > 0.0 {
            let instant = 1.0 / dt;
            self.fps = if self.fps == 0.0 { instant } else { self.fps + 0.05 * (instant - self.fps) };
        }
        self.draw_scene(r);
        let events = {
            let _p = profiler::scope("gui");
            self.draw_controls(input, r, audio)
        };
        debug_overlay::draw(r, self.window_width, dt);
        events
    }

    fn resize(&mut self, size: Vec2, measure: &dyn TextMeasure) {
        if self.window_width != size.x {
            self.window_width = size.x;
            self.piano_state.resize(size.x, measure);
            self.note_dim = self.piano_state.piano_props.white_key_width;
            self.key_spacing = self.piano_state.piano_props.key_spacing;
            self.font_size = ui_font_size(size.x);
            let (font_size_3, font_size_2) =
                notes::calculate_note_block_font_sizes(self.note_dim, measure);
            self.font_size_3 = font_size_3;
            self.font_size_2 = font_size_2;
        }
        self.window_height = size.y;
    }

    fn update(&mut self, input: &InputState, dt: f32) {
        if !self.player.is_paused {
            self.background_time += dt;
        }
        if input.space_pressed {
            self.player.space_pressed();
        }
        self.player.update(dt);

        self.controls_state.update_mouse_state(input.mouse_pos, dt);

        // Reset all key press states, then press the keys under the playhead.
        self.piano_state.reset_keys();
        if let Some(notes) = self.player.notes_at_playhead() {
            self.piano_state
                .trigger_key_press(notes, &self.player.instrument_colors);
        }

        self.controls_state
            .update_controls_panel_position(self.window_height);
        self.piano_state.update_key_animation(dt);
    }

    fn update_audio(&mut self, audio: &mut dyn AudioBackend) {
        let _p = profiler::scope("update_audio");
        if let Some(notes) = self.player.take_due_notes() {
            audio.play_tick(notes);
        }
    }

    fn draw_scene(&mut self, r: &mut dyn Renderer) {
        let _p = profiler::scope("draw");
        let size = Vec2::new(self.window_width, self.window_height);
        {
            let _p = profiler::scope("background");
            r.draw_background(self.background_time, size, &self.theme);
        }
        {
            let _p = profiler::scope("notes");
            self.draw_notes(r);
        }
        {
            let _p = profiler::scope("piano");
            self.draw_piano(r);
        }
        {
            let _p = profiler::scope("status");
            r.draw_text(
                &self.player.title,
                Vec2::new(10.0, 10.0),
                self.font_size,
                self.theme.text_color,
            );
        }
        self.draw_end_message(r);
        r.draw_text(
            &format!("{:.0} FPS", self.fps),
            Vec2::new(self.window_width - 100.0, 10.0),
            20.0,
            Rgba::new(0, 228, 48, 255),
        );
    }

    fn draw_notes(&self, r: &mut dyn Renderer) {
        let scene = NoteScene {
            window_width: self.window_width,
            window_height: self.window_height,
            keys: &self.piano_state.all_keys,
            key_map: &self.piano_state.key_map,
            piano_props: &self.piano_state.piano_props,
            note_blocks: &self.player.note_blocks,
            current_tick: self.player.current_tick,
            note_dim: self.note_dim,
            key_spacing: self.key_spacing,
            instrument_colors: &self.player.instrument_colors,
            font_size_2: self.font_size_2,
            font_size_3: self.font_size_3,
        };
        // Layout only needs the renderer for text measurement; collect, then draw.
        let mut sprites: Vec<(Rect, Rgba, &str, f32, Vec2)> = Vec::new();
        notes::visible_notes(&scene, &*r, |n| {
            sprites.push((n.rect, n.color, n.label, n.font_size, n.label_pos));
        });
        for (rect, color, label, font_size, pos) in sprites {
            r.draw_sprite(Sprite::Note, rect, color);
            r.draw_text(label, pos, font_size, Rgba::WHITE);
        }
    }

    fn draw_piano(&self, r: &mut dyn Renderer) {
        r.draw_rect(
            self.piano_state
                .bounds(self.window_width, self.window_height),
            Rgba::BLACK,
        );
        let mut sprites = Vec::with_capacity(88);
        piano::for_each_key_sprite(
            &self.piano_state,
            self.window_width,
            self.window_height,
            &self.theme,
            &*r,
            |k| sprites.push((k.rect, k.color, k.label, k.label_pos, k.font_size, k.text_color)),
        );
        for (rect, color, label, pos, font_size, text_color) in sprites {
            r.draw_sprite(Sprite::PianoKey, rect, color);
            r.draw_text(label, pos, font_size, text_color);
        }
    }

    fn draw_end_message(&self, r: &mut dyn Renderer) {
        if !self.player.is_end {
            return;
        }
        let (w, h) = (self.window_width, self.window_height);
        let accent = self.theme.accent_color;
        for (text, dy) in [(self.player.title.as_str(), -50.0), ("Press Space to Restart", 10.0)] {
            let width = r.measure_text(text, self.font_size).x;
            r.draw_text(text, Vec2::new(w / 2. - width / 2., h / 2. + dy), self.font_size, accent);
        }
    }

    fn draw_controls(
        &mut self,
        input: &InputState,
        r: &mut dyn Renderer,
        audio: &mut dyn AudioBackend,
    ) -> AppEvents {
        let layout = self
            .controls_state
            .layout(self.window_width, self.window_height);
        self.controls_state.hold_open_if_hovered(layout.panel);
        self.ui.begin_frame(input);

        let theme = self.theme.clone();
        let accent = theme.accent_color;
        let button_size = self.controls_state.button_size;
        let is_paused = self.player.is_paused;
        let mut events = AppEvents::default();

        r.draw_rect(layout.panel, Rgba::BLACK.alpha(0.7));

        // Play / pause
        if self.ui.button(ID_PLAY_PAUSE, layout.play_pause, &theme, r) {
            self.player.toggle_pause();
        }
        r.draw_sprite(
            if is_paused { Sprite::Play } else { Sprite::Pause },
            layout.play_pause,
            accent,
        );

        // Reset
        if self.ui.button(ID_RESET, layout.reset, &theme, r) {
            self.player.reset();
        }
        r.draw_sprite(Sprite::Reset, layout.reset, accent);

        // Timeline
        let mut tick = self.player.current_tick;
        if self.ui.slider(
            ID_TIMELINE,
            layout.timeline,
            &mut tick,
            0.0,
            self.player.song_length as f32,
            SliderStyle::Handle,
            &theme,
            r,
        ) {
            self.player.seek_to_tick(tick);
        }
        let time_text = format!(
            "{} / {}",
            utils::time_formatter(self.player.elapsed_time),
            utils::time_formatter(self.player.total_duration)
        );
        let text_h = r.measure_text(&time_text, self.font_size).y;
        r.draw_text(
            &time_text,
            Vec2::new(
                layout.timeline.x + 10.,
                layout.timeline.y + layout.timeline.h / 2.0 - text_h / 2.0,
            ),
            self.font_size,
            accent,
        );

        // Volume
        if self.ui.slider(
            ID_VOLUME,
            layout.volume,
            &mut self.volume,
            0.0,
            1.0,
            SliderStyle::Bar,
            &theme,
            r,
        ) {
            audio.set_master_volume(self.volume);
        }
        let volume_sprite = match self.volume {
            v if v == 0.0 => Sprite::Volume0,
            v if v <= 0.25 => Sprite::Volume25,
            v if v <= 0.5 => Sprite::Volume50,
            v if v <= 0.75 => Sprite::Volume75,
            _ => Sprite::Volume100,
        };
        r.draw_sprite(
            volume_sprite,
            Rect::new(
                layout.volume.x + button_size.x / 2.,
                layout.volume.y,
                button_size.x,
                button_size.y,
            ),
            accent.alpha(0.5),
        );

        // Fullscreen
        if self.ui.button(ID_FULLSCREEN, layout.fullscreen, &theme, r) {
            events.toggle_fullscreen = true;
        }

        // Big play button while paused
        if is_paused {
            if self.ui.button(ID_CENTER_PLAY, layout.center_play, &theme, r) {
                self.player.play();
            }
            r.draw_sprite(Sprite::Play, layout.center_play, accent);
        }

        r.draw_sprite(Sprite::Fullscreen, layout.fullscreen, accent);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::Instrument;
    use crate::notes::NoteBlock;
    use std::collections::HashMap;

    /// Records every draw call so tests can assert on what a frame produced.
    #[derive(Default)]
    struct Recorder {
        sprites: Vec<Sprite>,
        texts: Vec<String>,
        rects: usize,
    }

    impl TextMeasure for Recorder {
        fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
            Vec2::new(text.len() as f32 * font_size * 0.5, font_size)
        }
    }

    impl Renderer for Recorder {
        fn draw_background(&mut self, _: f32, _: Vec2, _: &Theme) {}
        fn draw_rect(&mut self, _: Rect, _: Rgba) {
            self.rects += 1;
        }
        fn draw_rect_outline(&mut self, _: Rect, _: f32, _: Rgba) {}
        fn draw_sprite(&mut self, s: Sprite, _: Rect, _: Rgba) {
            self.sprites.push(s);
        }
        fn draw_text(&mut self, t: &str, _: Vec2, _: f32, _: Rgba) {
            self.texts.push(t.to_string());
        }
    }

    #[derive(Default)]
    struct Audio {
        played: usize,
        volume: Option<f32>,
    }

    impl AudioBackend for Audio {
        fn load_instrument(&mut self, _: &Instrument) {}
        fn play(&mut self, _: &NoteBlock) {
            self.played += 1;
        }
        fn set_master_volume(&mut self, v: f32) {
            self.volume = Some(v);
        }
    }

    const SIZE: Vec2 = Vec2::new(1280.0, 720.0);

    fn app() -> App {
        let note = NoteBlock {
            instrument: 0,
            key: 60,
            frequency_ratio: 1.0,
            volume: 1.0,
            pan: 0.0,
        };
        // 100 ticks at 10 ticks/s, a note on every tick.
        let player = Player::new(
            "Song - Me".into(),
            1000,
            100,
            (0..100).map(|_| vec![note.clone()]).collect(),
            HashMap::new(),
        );
        App::new(Theme::default(), player, SIZE, 0.5, 0.0, &Recorder::default())
    }

    fn idle() -> InputState {
        InputState::default()
    }

    fn at(p: Vec2, down: bool, pressed: bool, released: bool) -> InputState {
        InputState {
            mouse_pos: p,
            mouse_down: down,
            mouse_pressed: pressed,
            mouse_released: released,
            ..Default::default()
        }
    }

    fn center(r: Rect) -> Vec2 {
        Vec2::new(r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn click(app: &mut App, audio: &mut Audio, target: Vec2) -> AppEvents {
        let mut r = Recorder::default();
        app.frame(&at(target, true, true, false), 0.016, SIZE, &mut r, audio);
        app.frame(&at(target, false, false, true), 0.016, SIZE, &mut r, audio)
    }

    #[test]
    fn starts_paused_with_big_play_button_and_no_sound() {
        let (mut a, mut audio, mut r) = (app(), Audio::default(), Recorder::default());
        a.frame(&idle(), 0.016, SIZE, &mut r, &mut audio);
        assert!(a.player.is_paused);
        assert_eq!(r.sprites.iter().filter(|s| **s == Sprite::Play).count(), 2);
        assert!(r.texts.iter().any(|t| t == "Song - Me"));
        assert_eq!(audio.played, 0);
    }

    #[test]
    fn clicking_center_play_starts_and_fires_notes() {
        let (mut a, mut audio) = (app(), Audio::default());
        let layout = a.controls_state.layout(SIZE.x, SIZE.y);
        click(&mut a, &mut audio, center(layout.center_play));
        assert!(!a.player.is_paused);

        let mut r = Recorder::default();
        for _ in 0..30 {
            a.frame(&idle(), 0.016, SIZE, &mut r, &mut audio);
        }
        assert!(audio.played >= 3, "played {}", audio.played);
        // the pause icon replaces the play icon in the panel
        assert!(r.sprites.contains(&Sprite::Pause));
    }

    #[test]
    fn space_toggles_playback() {
        let (mut a, mut audio, mut r) = (app(), Audio::default(), Recorder::default());
        let space = InputState {
            space_pressed: true,
            ..Default::default()
        };
        a.frame(&space, 0.016, SIZE, &mut r, &mut audio);
        assert!(!a.player.is_paused);
        a.frame(&space, 0.016, SIZE, &mut r, &mut audio);
        assert!(a.player.is_paused);
    }

    #[test]
    fn timeline_drag_seeks_and_volume_drag_reaches_audio() {
        let (mut a, mut audio) = (app(), Audio::default());
        let l = a.controls_state.layout(SIZE.x, SIZE.y);
        let mut r = Recorder::default();

        let mid = center(l.timeline);
        a.frame(&at(mid, true, true, false), 0.016, SIZE, &mut r, &mut audio);
        assert!(a.player.current_tick > 30.0 && a.player.current_tick < 70.0);

        let vol = Vec2::new(l.volume.x + l.volume.w * 0.9, l.volume.y + 5.0);
        a.frame(&at(vol, true, true, false), 0.016, SIZE, &mut r, &mut audio);
        assert!(audio.volume.unwrap() > 0.8);
        assert_eq!(a.volume, audio.volume.unwrap());
    }

    #[test]
    fn fullscreen_click_is_reported_to_the_frontend() {
        let (mut a, mut audio) = (app(), Audio::default());
        let l = a.controls_state.layout(SIZE.x, SIZE.y);
        let events = click(&mut a, &mut audio, center(l.fullscreen));
        assert!(events.toggle_fullscreen);
    }

    #[test]
    fn resize_relayouts_piano() {
        let (mut a, mut audio, mut r) = (app(), Audio::default(), Recorder::default());
        let before = a.piano_state.piano_props.white_key_width;
        a.frame(&idle(), 0.016, Vec2::new(800.0, 600.0), &mut r, &mut audio);
        assert!(a.piano_state.piano_props.white_key_width < before);
        assert_eq!(a.window_height, 600.0);
    }

    #[test]
    fn profiler_overlay_draws_when_enabled() {
        let (mut a, mut audio, mut r) = (app(), Audio::default(), Recorder::default());
        let toggle = InputState {
            toggle_profiler: true,
            ..Default::default()
        };
        a.frame(&toggle, 0.016, SIZE, &mut r, &mut audio);
        a.frame(&idle(), 0.016, SIZE, &mut r, &mut audio);
        a.frame(&idle(), 0.016, SIZE, &mut Recorder::default(), &mut audio);
        let mut last = Recorder::default();
        a.frame(&idle(), 0.016, SIZE, &mut last, &mut audio);
        assert!(last.texts.iter().any(|t| t.starts_with("frame ")));
        // leave the thread-local profiler off for other tests on this thread
        a.frame(&toggle, 0.016, SIZE, &mut r, &mut audio);
    }
}
