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

    let nbs_data = song::load_nbs_file(Some(&data));
    let (mut app_state, mut rl, thread) =
        app_state::AppState::setup_application(config.clone(), &nbs_data);

    /* Audio Engine */
    // audio_engine.set_global_volume(volume);
    let raylib_audio = RaylibAudio::init_audio_device().expect("Failed to initialize audio device");
    let mut audio_engine: audio::AudioEngine = audio::AudioEngine::new(
        &raylib_audio,
        Some(app_state.song_state.extra_sounds.clone()),
    );

    app_state.window_width = rl.get_screen_width() as f32;
    app_state.window_height = rl.get_screen_height() as f32;

    while !rl.window_should_close() {
        app_state.toggle_fullscreen(&mut rl);
        app_state.update_window_dimensions(&mut rl);
        let delta_time = rl.get_frame_time();
        app_state.update(&mut rl, delta_time);
        app_state.update_audio(&mut audio_engine);
        let mut d = rl.begin_drawing(&thread);
        app_state.draw(&mut d);
        app_state.update_and_draw_gui(&mut d, &raylib_audio);
    }
}
