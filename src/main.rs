extern crate raylib;
use nbs_player_core::audio::InstrumentBank;
use nbs_player_core::config::AppConfig;
use nbs_player_core::{notes, profiler, song};
use raylib::prelude::*;
use simple_logger::SimpleLogger;
use std::env;

mod app_state;
mod audio;
mod background;
mod font;
mod profiler_overlay;
mod render;
mod textures;
mod theme;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <json-config>", args[0]);
        std::process::exit(1);
    }

    // Try to parse the JSON argument
    let config: AppConfig = match serde_json::from_str(&args[1]) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = SimpleLogger::new().init() {
        eprintln!("Failed to initialize logger: {}", err);
        std::process::exit(1);
    }

    let data = match utils::load_file("song.nbsx") {
        Ok(data) => data,
        Err(err) => {
            log::error!("Error loading file: {}", err);
            std::process::exit(1);
        }
    };

    let song_data = match song::load_nbs_file(Some(&data)) {
        Ok(song_data) => song_data,
        Err(err) => {
            log::error!("Error loading song: {}", err);
            std::process::exit(1);
        }
    };

    /* Instruments and note events don't need a window or an audio device. */
    let bank = InstrumentBank::new(&song_data.extra_sounds);
    let note_blocks = notes::get_note_blocks(&song_data.song, &bank.base_keys());

    let (mut app_state, mut rl, thread) =
        app_state::AppState::setup_application(config.clone(), &song_data.song, note_blocks);

    /* Audio */
    let raylib_audio = audio::needs_raylib_audio(config.audio_backend).then(|| {
        RaylibAudio::init_audio_device().expect("Failed to initialize audio device")
    });
    let mut audio_backend =
        audio::create_backend(config.audio_backend, raylib_audio.as_ref(), &bank);
    audio_backend.set_master_volume(app_state.volume);

    app_state.window_width = rl.get_screen_width() as f32;
    app_state.window_height = rl.get_screen_height() as f32;

    // NBS_PROFILE_FRAMES=N: profile N frames from the start, print report, exit.
    let bench_frames: Option<u32> = env::var("NBS_PROFILE_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    if bench_frames.is_some() {
        profiler::toggle();
        app_state.player.play();
    }
    let mut frame_count = 0u32;

    while !rl.window_should_close() {
        if let Some(n) = bench_frames {
            if frame_count >= n {
                profiler::end_frame();
                profiler::log_report();
                profiler::dump_folded();
                break;
            }
            frame_count += 1;
        }
        profiler::end_frame();
        profiler_overlay::handle_input(&rl);
        {
            let _p = profiler::scope("update");
            app_state.toggle_fullscreen(&mut rl);
            app_state.update_window_dimensions(&mut rl);
            let delta_time = rl.get_frame_time();
            app_state.update(&mut rl, delta_time);
        }
        app_state.update_audio(audio_backend.as_mut());

        let mut d = rl.begin_drawing(&thread);

        app_state.draw(&mut d);
        {
            let _p = profiler::scope("gui");
            app_state.update_and_draw_gui(&mut d, audio_backend.as_mut());
        }
        let frame_time = d.get_frame_time();
        profiler_overlay::draw(&mut d, app_state.window_width, frame_time);
    }
}
