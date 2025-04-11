extern crate raylib;
use raylib::prelude::*;
use simple_logger::SimpleLogger;
use std::env;

mod app_state;
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

    let nbs_data = song::load_nbs_file(Some(&data));
    let (mut app_state, mut rl, thread) =
        app_state::AppState::setup_application(config.clone(), &nbs_data);

    /* Audio Engine */
    let raylib_audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio device");
    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(
        &raylib_audio,
        Some(app_state.song_state.extra_sounds.clone()),
    );

    app_state.window_width = rl.get_screen_width() as f32;
    app_state.window_height = rl.get_screen_height() as f32;
    app_state.song_state.note_blocks =
        note::get_note_blocks(&app_state.song_state.nbs_file, &audio_engine.sounds);

    let shader_header = {
        if cfg!(target_arch = "wasm32") {
            "#version 100\n\nprecision mediump float;\n"
        } else {
            "#version 330 core\n"
        }
    };

    // Simple shader implementation
    let fs_code = include_str!("../assets/shaders/plain_background.frag");
    let fs_code = format!("{}{}", shader_header, fs_code);

    let mut shader = rl.load_shader_from_memory(&thread, None, Some(&fs_code));
    let i_time_loc = shader.get_shader_location("iTime"); // float iTime;
    let i_resolution_loc = shader.get_shader_location("iResolution"); // vec2 iResolution;
    let background_color_loc = shader.get_shader_location("background_color"); // vec3 color1;
    let accent_color_loc = shader.get_shader_location("accent_color"); // vec3 color2;
    let speed_loc = shader.get_shader_location("speed"); // float speed;
    // get random number between 0.0 and 1.0
    let mut shader_time = rand::random::<f32>() * 1000.0;

    while !rl.window_should_close() {
        app_state.toggle_fullscreen(&mut rl);
        app_state.update_window_dimensions(&mut rl);
        let delta_time = rl.get_frame_time();
        shader_time += delta_time;
        app_state.update(&mut rl, delta_time, &audio_engine.sounds);
        app_state.update_audio(&mut audio_engine);

        let mut d = rl.begin_drawing(&thread);

        // Draw shader background if available
        {
            let resolution: [f32; 2] = [d.get_screen_width() as f32, d.get_screen_height() as f32];

            // Set shader uniforms
            shader.set_shader_value(i_time_loc, shader_time);
            shader.set_shader_value(i_resolution_loc, resolution);
            shader.set_shader_value(speed_loc, 0.5);
            shader.set_shader_value(
                background_color_loc,
                [
                    app_state.theme.background_color.r as f32 / 255.0,
                    app_state.theme.background_color.g as f32 / 255.0,
                    app_state.theme.background_color.b as f32 / 255.0,
                ],
            );
            shader.set_shader_value(
                accent_color_loc,
                [
                    app_state.theme.accent_color.r as f32 / 255.0,
                    app_state.theme.accent_color.g as f32 / 255.0,
                    app_state.theme.accent_color.b as f32 / 255.0,
                ],
            );

            let mut shader_mode_handle = d.begin_shader_mode(&mut shader);

            // Draw fullscreen rectangle to activate shader
            shader_mode_handle.draw_rectangle(
                0,
                0,
                shader_mode_handle.get_screen_width(),
                shader_mode_handle.get_screen_height(),
                Color::WHITE,
            );
        }

        app_state.draw(&mut d);
        app_state.update_and_draw_gui(&mut d, &raylib_audio);
    }
}
