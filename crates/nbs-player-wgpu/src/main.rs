//! Native entry point. Usage: `nbs-player-wgpu [json-config] [song-path]`.
//! With no arguments it plays `song.nbsx` if present, else the bundled demo song, using a
//! default theme.

use nbs_player_core::config::AppConfig;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .with_module_level("wgpu_hal", log::LevelFilter::Warn)
        .with_module_level("naga", log::LevelFilter::Warn)
        .init()
        .ok();

    let config = match std::env::args().nth(1) {
        Some(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("Error parsing JSON config: {e}");
            std::process::exit(1);
        }),
        None => AppConfig::demo(),
    };
    let path = std::env::args().nth(2).unwrap_or_else(|| "song.nbsx".into());
    let song = std::fs::read(&path).ok();

    if let Err(e) = nbs_player_wgpu::run_native(config, song) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
