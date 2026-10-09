//! macroquad frontend. Usage: `nbs-player-macroquad [json-config] [song-path]`.
//! With no arguments it plays `song.nbsx` if present, else the bundled demo song, using a
//! default theme. On wasm32 the song and config are fetched as `song.nbsx` / `config.json`.

use macroquad::prelude::*;
use nbs_player_core::app::App;
use nbs_player_core::audio::{AudioBackend, InstrumentBank};
#[cfg(not(target_arch = "wasm32"))]
use nbs_player_core::audio::NullBackend;
use nbs_player_core::config::{AppConfig, BackgroundType, FontID, ThemeConfig};
use nbs_player_core::player::Player;
use nbs_player_core::theme::Theme;
use nbs_player_core::types::Vec2 as CoreVec2;
use nbs_player_core::{notes, song};

mod render;

use render::MacroquadRenderer;

fn default_config() -> AppConfig {
    AppConfig {
        font_id: FontID::PixelPlay,
        background: BackgroundType::Plain,
        window_width: 1280,
        window_height: 720,
        theme: ThemeConfig {
            background_color: "#66BFFF".into(),
            accent_color: "#8A2BE2".into(),
            text_color: "#000000".into(),
            white_key_color: "#FFFFFF".into(),
            black_key_color: "#333333".into(),
            white_text_key_color: "#000000".into(),
            black_text_key_color: "#FFFFFF".into(),
        },
        initial_volume: None,
        target_fps: None,
        audio_backend: Default::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn config_text() -> Option<String> {
    std::env::args().nth(1)
}

#[cfg(target_arch = "wasm32")]
fn config_text() -> Option<String> {
    None
}

/// Native: the first CLI argument. Web: an optional `config.json` next to the page.
async fn load_config_text() -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        config_text()
    }
    #[cfg(target_arch = "wasm32")]
    {
        macroquad::file::load_string("config.json").await.ok()
    }
}

fn parse_config(text: Option<&str>) -> AppConfig {
    match text.map(serde_json::from_str) {
        Some(Ok(cfg)) => cfg,
        Some(Err(e)) => {
            macroquad::miniquad::error!("Error parsing JSON config: {}", e);
            std::process::exit(1);
        }
        None => default_config(),
    }
}

fn window_conf() -> Conf {
    let cfg = parse_config(config_text().as_deref());
    Conf {
        window_title: "NBS Player".into(),
        window_width: cfg.window_width as i32,
        window_height: cfg.window_height as i32,
        ..Default::default()
    }
}

async fn load_song_bytes() -> Option<Vec<u8>> {
    #[cfg(not(target_arch = "wasm32"))]
    let path = std::env::args().nth(2).unwrap_or_else(|| "song.nbsx".into());
    #[cfg(target_arch = "wasm32")]
    let path = "song.nbsx".to_string();
    macroquad::file::load_file(&path).await.ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn create_audio() -> Box<dyn AudioBackend> {
    match nbs_player_cpal::CpalBackend::new() {
        Ok(b) => Box::new(b),
        Err(e) => {
            macroquad::miniquad::error!("audio unavailable ({}); running silent", e);
            Box::new(NullBackend)
        }
    }
}

/// Web audio: the core mixer, pulled by a Web Audio `ScriptProcessorNode` (see
/// `web/nbs_audio.js`). cpal's wasm host needs wasm-bindgen, whose glue does not coexist
/// with miniquad's loader, so the output side is a small JS plugin instead.
#[cfg(target_arch = "wasm32")]
mod web_audio {
    use nbs_player_core::audio::{AudioBackend, SharedMixer};
    use std::sync::{Mutex, OnceLock};

    unsafe extern "C" {
        /// Creates the AudioContext and returns its sample rate.
        fn nbs_audio_init() -> u32;
    }

    static MIXER: OnceLock<SharedMixer> = OnceLock::new();
    static BUFFER: Mutex<Vec<f32>> = Mutex::new(Vec::new());

    pub fn create() -> Box<dyn AudioBackend> {
        let rate = unsafe { nbs_audio_init() };
        let mixer = SharedMixer::new(rate);
        let _ = MIXER.set(mixer.clone());
        Box::new(mixer)
    }

    /// Called by the audio plugin for every output block. Returns interleaved stereo.
    #[unsafe(no_mangle)]
    pub extern "C" fn nbs_audio_fill(frames: u32) -> *const f32 {
        let mut buf = BUFFER.lock().unwrap();
        buf.clear();
        buf.resize(frames as usize * 2, 0.0);
        if let Some(mixer) = MIXER.get() {
            mixer.mix_stereo(&mut buf);
        }
        buf.as_ptr()
    }
}

#[cfg(target_arch = "wasm32")]
fn create_audio() -> Box<dyn AudioBackend> {
    web_audio::create()
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = parse_config(load_config_text().await.as_deref());

    let bytes = load_song_bytes().await;
    let song_data = match song::load_nbs_file(bytes.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            macroquad::miniquad::error!("Error loading song: {}", e);
            std::process::exit(1);
        }
    };
    let header = &song_data.song.header;

    let bank = InstrumentBank::new(&song_data.extra_sounds);
    let note_blocks = notes::get_note_blocks(&song_data.song, &bank.base_keys());
    let name = String::from_utf8(header.song_name.clone()).unwrap_or_else(|_| "Unknown".into());
    let author = String::from_utf8(header.song_author.clone()).unwrap_or_else(|_| "Unknown".into());
    let player = Player::new(
        format!("{name} - {author}"),
        header.tempo,
        header.song_length as usize,
        note_blocks,
        notes::generate_instrument_palette(),
    );

    let mut renderer = match MacroquadRenderer::new(config.font_id.ttf_bytes(), &config.background)
    {
        Ok(r) => r,
        Err(e) => {
            macroquad::miniquad::error!("Error creating renderer: {}", e);
            std::process::exit(1);
        }
    };

    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    let mut app = App::new(
        Theme::from_theme_config(&config.theme),
        player,
        CoreVec2::new(screen_width(), screen_height()),
        config.initial_volume.unwrap_or(0.5),
        macroquad::rand::gen_range(0.0, 1000.0),
        &renderer,
    );

    let mut audio = create_audio();
    bank.install(audio.as_mut());
    audio.set_master_volume(app.volume);

    // NBS_SCREENSHOT=name.png: save frame 15 and exit (for visual checks).
    #[cfg(not(target_arch = "wasm32"))]
    let screenshot = std::env::var("NBS_SCREENSHOT").ok();
    #[cfg(not(target_arch = "wasm32"))]
    let mut frame = 0u32;

    loop {
        let input = render::read_input();
        let size = CoreVec2::new(screen_width(), screen_height());
        let events = app.frame(&input, get_frame_time(), size, &mut renderer, audio.as_mut());
        if events.toggle_fullscreen {
            // macroquad has no fullscreen getter; track it ourselves.
            static FULL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            let now = !FULL.fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
            set_fullscreen(now);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(path), 15) = (&screenshot, frame) {
            get_screen_data().export_png(path);
            break;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            frame += 1;
        }
        next_frame().await;
    }
}
