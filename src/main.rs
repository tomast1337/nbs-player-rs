extern crate raylib;
use nbs_player_core::app::App;
use nbs_player_core::audio::InstrumentBank;
use nbs_player_core::config::AppConfig;
use nbs_player_core::player::Player;
use nbs_player_core::theme::Theme;
use nbs_player_core::types::Vec2;
use nbs_player_core::{notes, profiler, song};
use raylib::prelude::*;
use simple_logger::SimpleLogger;
use std::env;

mod audio;
mod background;
mod font;
mod render;
mod textures;
mod utils;

use render::{RaylibRenderer, Resources};

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

    if config.window_width < 200 || config.window_height < 200 {
        log::error!("Error: Window dimensions are too small (minimum 200x200)");
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
    let header = &song_data.song.header;

    /* Instruments and note events don't need a window or an audio device. */
    let bank = InstrumentBank::new(&song_data.extra_sounds);
    let note_blocks = notes::get_note_blocks(&song_data.song, &bank.base_keys());

    let song_name = String::from_utf8(header.song_name.clone()).unwrap_or_else(|_| "Unknown".into());
    let song_author =
        String::from_utf8(header.song_author.clone()).unwrap_or_else(|_| "Unknown".into());
    let title = format!("{} - {}", song_name, song_author);

    let player = Player::new(
        title.clone(),
        header.tempo,
        header.song_length as usize,
        note_blocks,
        notes::generate_instrument_palette(),
    );

    /* Window and GPU resources */
    let (mut rl, thread) = raylib::init()
        .size(config.window_width as i32, config.window_height as i32)
        .title(&title)
        .build();
    rl.set_target_fps(config.target_fps.unwrap_or(60));

    let font = font::load_fonts(config.font_id.clone(), &mut rl, &thread);
    let mut resources = Resources::new(&mut rl, &thread, font, config.background.clone());

    let window_size = Vec2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    let mut app = App::new(
        Theme::from_theme_config(&config.theme),
        player,
        window_size,
        config.initial_volume.unwrap_or(0.5),
        rand::random::<f32>() * 1000.0,
        &resources,
    );

    /* Audio */
    let raylib_audio = audio::needs_raylib_audio(config.audio_backend).then(|| {
        RaylibAudio::init_audio_device().expect("Failed to initialize audio device")
    });
    let mut audio_backend =
        audio::create_backend(config.audio_backend, raylib_audio.as_ref(), &bank);
    audio_backend.set_master_volume(app.volume);

    // NBS_PROFILE_FRAMES=N: profile N frames from the start, print report, exit.
    let bench_frames: Option<u32> = env::var("NBS_PROFILE_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    if bench_frames.is_some() {
        profiler::toggle();
        app.player.play();
    }
    // NBS_SCREENSHOT=name.png: save frame 15 (raylib writes it next to the executable).
    let screenshot = env::var("NBS_SCREENSHOT").ok();
    let mut frame_count = 0u32;

    while !rl.window_should_close() {
        if let Some(n) = bench_frames {
            if frame_count >= n {
                profiler::end_frame();
                profiler::log_report();
                profiler::dump_folded();
                break;
            }
        }
        if let (Some(path), 15) = (&screenshot, frame_count) {
            rl.take_screenshot(&thread, path);
            break;
        }
        frame_count += 1;

        let input = render::read_input(&rl);
        let dt = rl.get_frame_time();
        let size = Vec2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32);

        let events = {
            let mut d = rl.begin_drawing(&thread);
            let mut renderer = RaylibRenderer {
                d: &mut d,
                res: &mut resources,
            };
            app.frame(&input, dt, size, &mut renderer, audio_backend.as_mut())
        };

        if events.toggle_fullscreen {
            rl.toggle_fullscreen();
        }
    }
}
