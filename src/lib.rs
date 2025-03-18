// use macroquad::{
//    self, color,
//    input::{KeyCode, MouseButton, is_key_pressed, is_mouse_button_pressed},
//    text::{TextParams, draw_text_ex, load_ttf_font_from_bytes, measure_text},
//    time::{get_fps, get_frame_time},
//    window::{self, clear_background, request_new_screen_size},
// };

use glow::{self};
use log;
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};
use web_sys;

mod audio;
// mod font;
// mod note;
// mod piano;
mod song;
mod utils;

struct Context {
    gl: glow::Context,
    width: f32,
    height: f32,
}

impl Context {
    fn new(canvas_id: &str, width: f32, height: f32) -> Self {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        let mut attrs = web_sys::WebGlContextAttributes::new();
        attrs.set_stencil(true);
        attrs.set_antialias(false);

        let webgl2_context = canvas
            .get_context_with_context_options("webgl2", &attrs)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        let gl = glow::Context::from_webgl2_context(webgl2_context);

        Self { gl, width, height }
    }
}

#[wasm_bindgen]
pub async fn run(
    window_width: Option<f32>,
    window_height: Option<f32>,
    nbs_file: Option<Vec<u8>>,
    canvas_id: Option<String>,
) {
    console_log::init_with_level(log::Level::Debug).unwrap();
    console_error_panic_hook::set_once();

    let window_width = window_width.unwrap_or(1280.);
    let window_height = window_height.unwrap_or(720.);
    let canvas_id = canvas_id.unwrap_or("canvas".to_string());

    log::debug!("Canvas ID: {}", canvas_id);
    log::debug!("Window Width: {}", window_width);
    log::debug!("Window Height: {}", window_height);

    let nbs_data = song::load_nbs_file(nbs_file.as_deref());

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

    log::debug!("Song Title: {}", title);
    log::debug!("Notes Per Second: {}", notes_per_second);
    log::debug!("Total Duration: {}", total_duration);

    /*
    let (mut all_keys, key_map) = piano::generate_piano_keys();

    let mut piano_props;
    let mut note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);

    let note_texture = note::load_note_texture();
    note_texture.set_filter(macroquad::texture::FilterMode::Nearest);

    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(Some(extra_sounds), 0.5);

    let mut current_tick: f32; // Current tick in the song (now a float for sub-ticks)
    let mut elapsed_time: f32 = 0.; // Elapsed time in seconds

    let mut note_dim;
    let mut key_spacing; // Spacing between keys

    let mut played_ticks: Vec<bool> = vec![false; nbs_file.header.song_length as usize];

    let instrument_colors = note::generate_instrument_palette();

    let mut is_paused: bool = true;

    let font_data = include_bytes!("../assets/fonts/Monocraft.ttf");
    let mut font = load_ttf_font_from_bytes(font_data).unwrap();
    font.set_filter(macroquad::texture::FilterMode::Nearest);
    crate::font::FONT.set(font.clone()).unwrap();

    window_width = window::screen_width();
    window_height = window::screen_height();
    piano_props = piano::initialize_piano_dimensions(window_width, &all_keys);
    note_dim = piano_props.white_key_width;
    key_spacing = piano_props.key_spacing;

    loop {
        if window_width != window::screen_width() {
            window_width = window::screen_width();
            piano_props = piano::initialize_piano_dimensions(window_width, &all_keys);
            note_dim = piano_props.white_key_width;
            key_spacing = piano_props.key_spacing;
        }
        if window_height != window::screen_height() {
            window_height = window::screen_height();
        }

        let delta_time = get_frame_time();

        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
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

        // Calculate font size based on screen width with min and max limits
        let min_font_size = 20;
        let max_font_size = 40;
        let font_size =
            (window_width / 64.0).clamp(min_font_size as f32, max_font_size as f32) as u16;

        // Define text positions
        let start_x = 10.0;
        let mut start_y = 30.0;
        let line_height = 20.0; // Space between lines

        // Define text color
        let text_color = color::BLACK;

        // Draw song status
        let fps = get_fps();
        let current_tick_text = format!("Current Tick: {:.4}", current_tick);
        let notes_rendered_text = format!("Notes Rendered: {}", notes_rendered);
        let duration_text = format!(
            "Duration: {}|{}",
            time_formatter(elapsed_time),
            time_formatter(total_duration)
        );

        let text_parameters = TextParams {
            font_size,
            font: Some(&font),
            color: text_color,
            font_scale: 0.5,
            ..Default::default()
        };

        // Draw title
        draw_text_ex(&title, start_x, start_y, text_parameters.clone());

        // Draw duration
        start_y += line_height;
        draw_text_ex(&duration_text, start_x, start_y, text_parameters.clone());

        // Draw notes rendered
        start_y += line_height;
        draw_text_ex(
            &notes_rendered_text,
            start_x,
            start_y,
            text_parameters.clone(),
        );

        // Draw current tick
        start_y += line_height;
        draw_text_ex(
            &current_tick_text,
            start_x,
            start_y,
            text_parameters.clone(),
        );

        // Draw FPS in the top-right corner
        let fps_text = format!("FPS: {:.2}", fps);
        let fps_text_width = measure_text(&fps_text, Some(&font), font_size, 1.0).width;
        draw_text_ex(
            &fps_text,
            window_width - fps_text_width - 10.0, // 10.0 padding from the right edge
            15.0,
            text_parameters.clone(),
        );

        let is_end = elapsed_time >= total_duration;

        // Draw pause state
        if is_paused && !is_end {
            draw_text_ex(
                "Paused",
                window_width / 2. - 50.,
                window_height / 2.,
                TextParams {
                    font_size: 40,
                    font: Some(&font),
                    color: color::RED,
                    ..Default::default()
                },
            );
        }

        if is_end {
            draw_text_ex(
                "End of Song",
                window_width / 2. - 50.,
                window_height / 2.,
                TextParams {
                    font_size: 40,
                    font: Some(&font),
                    color: color::RED,
                    ..Default::default()
                },
            );
            // draw title
            draw_text_ex(
                &title,
                window_width / 2. - 50.,
                window_height / 2. + 50.,
                TextParams {
                    font_size: 20,
                    font: Some(&font),
                    color: color::BLACK,
                    ..Default::default()
                },
            );

            // draw press space to restart
            draw_text_ex(
                "Press Space to Restart",
                window_width / 2. - 50.,
                window_height / 2. + 100.,
                TextParams {
                    font_size: 20,
                    font: Some(&font),
                    color: color::BLACK,
                    ..Default::default()
                },
            );
        }

        window::next_frame().await
    }

     */
}
